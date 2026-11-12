use crate::config::Config;
use crate::discord_ipc::{Activity, ActivityAssets, ActivityTimestamps, DiscordClient};
use crate::git::{get_git_info, GitInfo};
use crate::languages::get_language_info;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct ActiveDoc {
    pub path: PathBuf,
    pub filename: String,
    pub language_id: String,
    pub line_count: usize,
    pub last_activity: Instant,
}

pub struct RpcBackend {
    #[allow(dead_code)]
    pub client: Client,
    pub config: Arc<RwLock<Config>>,
    pub discord: Arc<Mutex<Option<DiscordClient>>>,
    pub workspace_root: Arc<RwLock<Option<PathBuf>>>,
    pub active_doc: Arc<RwLock<Option<ActiveDoc>>>,
    pub is_idle: Arc<AtomicBool>,
    pub session_start: u64,
}

impl RpcBackend {
    pub fn new(client: Client) -> Self {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            client,
            config: Arc::new(RwLock::new(Config::default())),
            discord: Arc::new(Mutex::new(None)),
            workspace_root: Arc::new(RwLock::new(None)),
            active_doc: Arc::new(RwLock::new(None)),
            is_idle: Arc::new(AtomicBool::new(false)),
            session_start: start,
        }
    }

    async fn ensure_discord_connected(&self) -> bool {
        let mut discord_guard = self.discord.lock().await;
        if discord_guard.is_some() {
            return true;
        }

        let app_id = self.config.read().await.application_id.clone();
        match DiscordClient::connect(&app_id).await {
            Ok(client) => {
                *discord_guard = Some(client);
                eprintln!("[zed-rpc] Connected to Discord IPC");
                true
            }
            Err(e) => {
                eprintln!("[zed-rpc] Could not connect to Discord: {e}");
                false
            }
        }
    }

    pub async fn update_presence(&self) {
        if !self.ensure_discord_connected().await {
            return;
        }

        let config = self.config.read().await.clone();
        let idle = self.is_idle.load(Ordering::Relaxed);
        let maybe_doc = self.active_doc.read().await.as_ref().map(|d| {
            (
                d.path.clone(),
                d.filename.clone(),
                d.language_id.clone(),
                d.line_count,
            )
        });
        let maybe_ws = self.workspace_root.read().await.clone();

        let (workspace_name, git_info) = if let Some(ref doc_path) = maybe_doc.as_ref().map(|d| &d.0) {
            let info = get_git_info(doc_path);
            let ws = info
                .repo_name
                .clone()
                .or_else(|| doc_path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).map(String::from))
                .unwrap_or_else(|| "Zed".to_string());
            (ws, info)
        } else if let Some(ref ws_path) = maybe_ws {
            let info = get_git_info(ws_path);
            let ws = ws_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from)
                .unwrap_or_else(|| "Zed".to_string());
            (ws, info)
        } else {
            ("Zed".to_string(), GitInfo::default())
        };

        let branch_str = git_info
            .branch
            .as_deref()
            .filter(|_| config.show_git)
            .map(|b| format!(" ({b})"))
            .unwrap_or_default();

        let branch_clean = git_info.branch.as_deref().unwrap_or("none");

        let activity = if idle {
            Activity {
                details: Some(format!("Idling in {workspace_name}")),
                state: Some("Zzz...".to_string()),
                timestamps: if config.show_timestamps {
                    Some(ActivityTimestamps {
                        start: Some(self.session_start),
                    })
                } else {
                    None
                },
                assets: Some(ActivityAssets {
                    large_image: Some("zed".to_string()),
                    large_text: Some("Zed Editor (Idle)".to_string()),
                    small_image: None,
                    small_text: None,
                }),
            }
        } else if let Some((_, filename, language_id, line_count)) = maybe_doc {
            let lang_info = get_language_info(&language_id, &filename);

            let mut details = config
                .details_format
                .replace("{filename}", &filename)
                .replace("{language}", lang_info.display_name)
                .replace("{workspace}", &workspace_name)
                .replace("{branch}", branch_clean);

            if config.show_line_count && line_count > 0 && !details.contains("line") {
                details = format!("{details} ({line_count} lines)");
            }

            let mut state = config
                .state_format
                .replace("{workspace}", &format!("{workspace_name}{branch_str}"))
                .replace("{filename}", &filename)
                .replace("{language}", lang_info.display_name)
                .replace("{branch}", branch_clean);

            if state.is_empty() {
                state = format!("{workspace_name}{branch_str}");
            }

            let large_icon_url = format!("{}{}.png", config.base_icons_url, lang_info.icon_slug);

            Activity {
                details: Some(details),
                state: Some(state),
                timestamps: if config.show_timestamps {
                    Some(ActivityTimestamps {
                        start: Some(self.session_start),
                    })
                } else {
                    None
                },
                assets: Some(ActivityAssets {
                    large_image: Some(large_icon_url),
                    large_text: Some(lang_info.display_name.to_string()),
                    small_image: Some("zed".to_string()),
                    small_text: Some("Zed Editor".to_string()),
                }),
            }
        } else {
            Activity {
                details: Some(format!("Workspace: {workspace_name}{branch_str}")),
                state: Some("Exploring".to_string()),
                timestamps: if config.show_timestamps {
                    Some(ActivityTimestamps {
                        start: Some(self.session_start),
                    })
                } else {
                    None
                },
                assets: Some(ActivityAssets {
                    large_image: Some("zed".to_string()),
                    large_text: Some("Zed Editor".to_string()),
                    small_image: None,
                    small_text: None,
                }),
            }
        };

        let mut discord_guard = self.discord.lock().await;
        if let Some(ref mut client) = *discord_guard {
            if let Err(e) = client.set_activity(Some(activity)).await {
                eprintln!("[zed-rpc] Failed to set activity: {e}");
                *discord_guard = None; // Reset so next call will reconnect
            }
        }
    }

    pub fn start_idle_and_reconnect_worker(self: &Arc<Self>) {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;

                let idle_timeout = this.config.read().await.idle_timeout_seconds;
                let is_idle_now = {
                    let doc_guard = this.active_doc.read().await;
                    if let Some(ref doc) = *doc_guard {
                        doc.last_activity.elapsed() > Duration::from_secs(idle_timeout)
                    } else {
                        false
                    }
                };

                let was_idle = this.is_idle.load(Ordering::Relaxed);
                if is_idle_now && !was_idle {
                    this.is_idle.store(true, Ordering::Relaxed);
                    this.update_presence().await;
                } else if !this.ensure_discord_connected().await {
                    // Try reconnect if disconnected
                }
            }
        });
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for RpcBackend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        eprintln!("[zed-rpc] Initializing LSP server");

        // Load configuration from initialization_options
        let new_config = Config::from_value(params.initialization_options);
        *self.config.write().await = new_config;

        // Determine workspace root
        if let Some(folders) = params.workspace_folders {
            if let Some(first) = folders.into_iter().next() {
                if let Ok(path) = first.uri.to_file_path() {
                    *self.workspace_root.write().await = Some(path);
                }
            }
        } else if let Some(root_uri) = params.root_uri {
            if let Ok(path) = root_uri.to_file_path() {
                *self.workspace_root.write().await = Some(path);
            }
        }

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::FULL,
            )),
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(true)),
                }),
                file_operations: None,
            }),
            ..Default::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "Zed Discord RPC".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        eprintln!("[zed-rpc] LSP server initialized");
        self.update_presence().await;
    }

    async fn shutdown(&self) -> Result<()> {
        eprintln!("[zed-rpc] Shutting down LSP server");
        let mut discord_guard = self.discord.lock().await;
        if let Some(ref mut client) = *discord_guard {
            let _ = client.close().await;
        }
        *discord_guard = None;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let Ok(path) = uri.to_file_path() else { return };
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let line_count = params.text_document.text.lines().count();

        *self.active_doc.write().await = Some(ActiveDoc {
            path,
            filename,
            language_id: params.text_document.language_id,
            line_count,
            last_activity: Instant::now(),
        });

        self.is_idle.store(false, Ordering::Relaxed);
        self.update_presence().await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut doc_guard = self.active_doc.write().await;
        if let Some(ref mut doc) = *doc_guard {
            doc.last_activity = Instant::now();
            if let Some(change) = params.content_changes.into_iter().next() {
                let lines = change.text.lines().count();
                if lines > 0 {
                    doc.line_count = lines;
                }
            }
        }
        drop(doc_guard);

        if self.is_idle.swap(false, Ordering::Relaxed) {
            self.update_presence().await;
        }
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        if let Some(ref mut doc) = *self.active_doc.write().await {
            doc.last_activity = Instant::now();
        }
        if self.is_idle.swap(false, Ordering::Relaxed) {
            self.update_presence().await;
        }
    }

    async fn did_close(&self, _params: DidCloseTextDocumentParams) {
        // Keep current presence or let idle kick in
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        if let Some(added) = params.event.added.into_iter().next() {
            if let Ok(path) = added.uri.to_file_path() {
                *self.workspace_root.write().await = Some(path);
                self.update_presence().await;
            }
        }
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let new_config = Config::from_value(Some(params.settings));
        *self.config.write().await = new_config;
        self.update_presence().await;
    }
}
