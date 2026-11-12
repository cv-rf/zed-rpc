use serde::Serialize;
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use uuid::Uuid;

const OPCODE_HANDSHAKE: u32 = 0;
const OPCODE_FRAME: u32 = 1;
const OPCODE_CLOSE: u32 = 2;

#[derive(Debug, Clone, Serialize)]
pub struct Activity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<ActivityTimestamps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<ActivityAssets>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityTimestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityAssets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

pub struct DiscordClient {
    stream: UnixStream,
    #[allow(dead_code)]
    pub application_id: String,
}

impl DiscordClient {
    pub async fn connect(application_id: &str) -> Result<Self, String> {
        let socket_path = find_ipc_socket().ok_or("No Discord IPC socket found")?;
        let mut stream = UnixStream::connect(&socket_path)
            .await
            .map_err(|e| format!("Failed to connect to Discord IPC socket at {socket_path:?}: {e}"))?;

        // Send handshake
        let handshake_data = serde_json::json!({
            "v": 1,
            "client_id": application_id
        });

        send_packet(&mut stream, OPCODE_HANDSHAKE, &handshake_data).await?;

        // Read handshake response
        let (_resp_op, _resp_val) = read_packet(&mut stream).await?;

        Ok(Self {
            stream,
            application_id: application_id.to_string(),
        })
    }

    pub async fn set_activity(&mut self, activity: Option<Activity>) -> Result<(), String> {
        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": activity
            },
            "nonce": Uuid::new_v4().to_string()
        });

        send_packet(&mut self.stream, OPCODE_FRAME, &payload).await?;
        let (_op, _resp) = read_packet(&mut self.stream).await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), String> {
        let payload = serde_json::json!({});
        let _ = send_packet(&mut self.stream, OPCODE_CLOSE, &payload).await;
        let _ = self.stream.shutdown().await;
        Ok(())
    }
}

fn find_ipc_socket() -> Option<PathBuf> {
    let mut search_paths = Vec::new();

    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        for i in 0..10 {
            search_paths.push(PathBuf::from(format!("{runtime_dir}/discord-ipc-{i}")));
        }
        search_paths.push(PathBuf::from(format!(
            "{runtime_dir}/app/com.discordapp.Discord/discord-ipc-0"
        )));
    }

    if let Ok(tmp) = env::var("TMPDIR") {
        for i in 0..10 {
            search_paths.push(PathBuf::from(format!("{tmp}/discord-ipc-{i}")));
        }
    }

    for i in 0..10 {
        search_paths.push(PathBuf::from(format!("/tmp/discord-ipc-{i}")));
    }

    search_paths.into_iter().find(|p| p.exists())
}

async fn send_packet(stream: &mut UnixStream, opcode: u32, data: &Value) -> Result<(), String> {
    let payload = serde_json::to_vec(data).map_err(|e| e.to_string())?;
    let len = payload.len() as u32;

    let mut header = [0u8; 8];
    header[0..4].copy_from_slice(&opcode.to_le_bytes());
    header[4..8].copy_from_slice(&len.to_le_bytes());

    stream
        .write_all(&header)
        .await
        .map_err(|e| format!("Failed to write IPC header: {e}"))?;
    stream
        .write_all(&payload)
        .await
        .map_err(|e| format!("Failed to write IPC payload: {e}"))?;

    Ok(())
}

async fn read_packet(stream: &mut UnixStream) -> Result<(u32, Value), String> {
    let mut header = [0u8; 8];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut header))
        .await
        .map_err(|_| "Discord IPC read timed out".to_string())?
        .map_err(|e| format!("Failed to read IPC header: {e}"))?;

    let opcode = u32::from_le_bytes(header[0..4].try_into().unwrap());
    let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;

    let mut buf = vec![0u8; len];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut buf))
        .await
        .map_err(|_| "Discord IPC payload read timed out".to_string())?
        .map_err(|e| format!("Failed to read IPC payload: {e}"))?;

    let val = serde_json::from_slice(&buf).map_err(|e| format!("Invalid JSON from Discord: {e}"))?;
    Ok((opcode, val))
}
