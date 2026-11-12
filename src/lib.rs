use std::fs;
use zed_extension_api::{self as zed, Result};

struct DiscordRpcExtension {
    cached_binary: Option<String>,
}

impl DiscordRpcExtension {
    fn resolve_binary(&mut self, worktree: &zed::Worktree) -> Result<String> {
        if let Some(ref path) = self.cached_binary {
            if fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
                return Ok(path.clone());
            }
        }

        let binary_name = if cfg!(target_os = "windows") {
            "discord-rpc-companion.exe"
        } else {
            "discord-rpc-companion"
        };

        // 1. Check if the binary is in PATH via worktree.which
        if let Some(path) = worktree.which(binary_name) {
            self.cached_binary = Some(path.clone());
            return Ok(path);
        }

        // 2. Check local directories
        let local_candidates = [
            binary_name.to_string(),
            format!("companion/target/release/{binary_name}"),
            format!("companion/target/debug/{binary_name}"),
            format!("target/release/{binary_name}"),
            format!("target/debug/{binary_name}"),
        ];

        for candidate in local_candidates {
            if fs::metadata(&candidate).map(|m| m.is_file()).unwrap_or(false) {
                let full = candidate;
                self.cached_binary = Some(full.clone());
                return Ok(full);
            }
        }

        // 3. Fallback to binary name so PATH resolution is attempted
        Ok(binary_name.to_string())
    }
}

impl zed::Extension for DiscordRpcExtension {
    fn new() -> Self {
        Self {
            cached_binary: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if language_server_id.as_ref() != "discord_rpc" {
            return Err(format!(
                "Unexpected language server: {}",
                language_server_id.as_ref()
            ));
        }

        let binary_path = self.resolve_binary(worktree)?;

        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(DiscordRpcExtension);
