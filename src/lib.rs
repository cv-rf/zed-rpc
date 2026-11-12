use std::fs;
use zed_extension_api::{self as zed, Result};

// Replace with your GitHub repository (e.g. "username/zed-rpc")
const REPO_NAME: &str = "atticl/zed-rpc";

struct DiscordRpcExtension {
    cached_binary: Option<String>,
}

impl DiscordRpcExtension {
    fn resolve_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        let binary_name = if cfg!(target_os = "windows") {
            "discord-rpc-companion.exe"
        } else {
            "discord-rpc-companion"
        };

        // 1. Check in-memory cached path
        if let Some(ref path) = self.cached_binary {
            if fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
                return Ok(path.clone());
            }
        }

        // 2. Check if the binary is in worktree PATH
        if let Some(path) = worktree.which(binary_name) {
            self.cached_binary = Some(path.clone());
            return Ok(path);
        }

        // 3. Check local build/install directories (useful for development)
        let local_candidates = [
            binary_name.to_string(),
            format!("companion/target/release/{binary_name}"),
            format!("companion/target/debug/{binary_name}"),
            format!("target/release/{binary_name}"),
            format!("target/debug/{binary_name}"),
        ];

        for candidate in local_candidates {
            if fs::metadata(&candidate).map(|m| m.is_file()).unwrap_or(false) {
                self.cached_binary = Some(candidate.clone());
                return Ok(candidate);
            }
        }

        // 4. Download pre-compiled companion binary from GitHub Release
        if let Ok(downloaded_path) = self.download_companion(language_server_id) {
            self.cached_binary = Some(downloaded_path.clone());
            return Ok(downloaded_path);
        }

        // 5. Fallback to binary name so PATH resolution is attempted
        Ok(binary_name.to_string())
    }

    fn download_companion(&self, language_server_id: &zed::LanguageServerId) -> Result<String> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            REPO_NAME,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let arch_str = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X8664 => "x86_64",
            _ => return Err(format!("Unsupported architecture: {arch:?}")),
        };

        let (os_str, ext_str, file_type) = match platform {
            zed::Os::Mac => ("apple-darwin", "tar.gz", zed::DownloadedFileType::GzipTar),
            zed::Os::Linux => ("unknown-linux-gnu", "tar.gz", zed::DownloadedFileType::GzipTar),
            zed::Os::Windows => ("pc-windows-msvc", "zip", zed::DownloadedFileType::Zip),
        };

        let asset_name = format!("discord-rpc-companion-{arch_str}-{os_str}.{ext_str}");

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("No release asset found matching {asset_name}"))?;

        let version_dir = format!("discord-rpc-companion-{}", release.version);
        let binary_path = match platform {
            zed::Os::Windows => format!("{version_dir}/discord-rpc-companion.exe"),
            _ => format!("{version_dir}/discord-rpc-companion"),
        };

        if !fs::metadata(&binary_path).map(|m| m.is_file()).unwrap_or(false) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(&asset.download_url, &version_dir, file_type)
                .map_err(|e| format!("Failed to download {asset_name}: {e}"))?;

            let _ = zed::make_file_executable(&binary_path);
        }

        Ok(binary_path)
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

        let binary_path = self.resolve_binary(language_server_id, worktree)?;

        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(DiscordRpcExtension);
