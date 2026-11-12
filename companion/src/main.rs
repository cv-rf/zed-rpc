mod config;
mod discord_ipc;
mod git;
mod languages;
mod lsp;

use config::DEFAULT_APP_ID;
use discord_ipc::{Activity, ActivityAssets, ActivityTimestamps, DiscordClient};
use git::get_git_info;
use lsp::RpcBackend;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Mode 1: --test (Verify Discord IPC connection and show presence)
    if args.iter().any(|arg| arg == "--test") {
        run_test_mode().await;
        return;
    }

    // Mode 2: --standalone (Run standalone monitor without Zed LSP)
    if args.iter().any(|arg| arg == "--standalone") {
        run_standalone_mode().await;
        return;
    }

    // Mode 3: Default LSP server for Zed editor (stdin/stdout)
    run_lsp_server().await;
}

async fn run_test_mode() {
    println!("\x1b[1;36m==> Testing Discord RPC connection...\x1b[0m");

    let app_id = env::var("DISCORD_APP_ID").unwrap_or_else(|_| DEFAULT_APP_ID.to_string());
    match DiscordClient::connect(&app_id).await {
        Ok(mut client) => {
            println!("\x1b[1;32m✓ Successfully connected to Discord IPC!\x1b[0m");

            let current_dir = env::current_dir().unwrap_or_default();
            let git_info = get_git_info(&current_dir);
            let workspace = git_info
                .repo_name
                .or_else(|| current_dir.file_name().and_then(|n| n.to_str()).map(String::from))
                .unwrap_or_else(|| "zed-rpc".to_string());
            let branch = git_info.branch.map(|b| format!(" ({b})")).unwrap_or_default();

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let activity = Activity {
                details: Some("Testing Zed Discord RPC Plugin".to_string()),
                state: Some(format!("Workspace: {workspace}{branch}")),
                timestamps: Some(ActivityTimestamps { start: Some(now) }),
                assets: Some(ActivityAssets {
                    large_image: Some("zed".to_string()),
                    large_text: Some("Zed Editor".to_string()),
                    small_image: Some("https://raw.githubusercontent.com/xhyrom/zed-discord-presence/main/assets/icons/rust.png".to_string()),
                    small_text: Some("Rust".to_string()),
                }),
            };

            if let Err(e) = client.set_activity(Some(activity)).await {
                eprintln!("\x1b[1;31m✗ Failed to set Discord activity: {e}\x1b[0m");
                return;
            }

            println!("\x1b[1;32m✓ Presence active! Check your Discord profile.\x1b[0m");
            println!("Keeping test presence active for 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;

            let _ = client.close().await;
            println!("Test completed successfully.");
        }
        Err(e) => {
            eprintln!("\x1b[1;31m✗ Could not connect to Discord: {e}\x1b[0m");
            eprintln!("Make sure Discord desktop client is running.");
        }
    }
}

async fn run_standalone_mode() {
    println!("\x1b[1;36m==> Starting Zed Discord RPC in standalone mode...\x1b[0m");
    let app_id = env::var("DISCORD_APP_ID").unwrap_or_else(|_| DEFAULT_APP_ID.to_string());

    let mut client = loop {
        match DiscordClient::connect(&app_id).await {
            Ok(c) => {
                println!("\x1b[1;32m✓ Connected to Discord IPC\x1b[0m");
                break c;
            }
            Err(e) => {
                eprintln!("Waiting for Discord... ({e})");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    };

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let cwd = env::current_dir().unwrap_or_default();
    let git_info = get_git_info(&cwd);
    let ws = git_info
        .repo_name
        .or_else(|| cwd.file_name().and_then(|n| n.to_str()).map(String::from))
        .unwrap_or_else(|| "Workspace".to_string());
    let branch = git_info.branch.map(|b| format!(" ({b})")).unwrap_or_default();

    let activity = Activity {
        details: Some(format!("Workspace: {ws}{branch}")),
        state: Some("Coding in Zed".to_string()),
        timestamps: Some(ActivityTimestamps {
            start: Some(start_time),
        }),
        assets: Some(ActivityAssets {
            large_image: Some("zed".to_string()),
            large_text: Some("Zed Editor".to_string()),
            small_image: None,
            small_text: None,
        }),
    };

    let _ = client.set_activity(Some(activity)).await;
    println!("Presence active. Press Ctrl+C to stop.");

    tokio::signal::ctrl_c().await.ok();
    let _ = client.close().await;
}

async fn run_lsp_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        let backend = Arc::new(RpcBackend::new(client));
        backend.start_idle_and_reconnect_worker();
        backend
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
