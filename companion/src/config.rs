use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_APP_ID: &str = "1263505205522337886";
pub const DEFAULT_ICONS_URL: &str =
    "https://raw.githubusercontent.com/xhyrom/zed-discord-presence/main/assets/icons/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub application_id: String,
    pub details_format: String,
    pub state_format: String,
    pub show_git: bool,
    pub show_line_count: bool,
    pub show_timestamps: bool,
    pub idle_timeout_seconds: u64,
    pub base_icons_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            application_id: DEFAULT_APP_ID.to_string(),
            details_format: "Editing {filename}".to_string(),
            state_format: "{workspace}".to_string(),
            show_git: true,
            show_line_count: true,
            show_timestamps: true,
            idle_timeout_seconds: 300,
            base_icons_url: DEFAULT_ICONS_URL.to_string(),
        }
    }
}

impl Config {
    pub fn from_value(value: Option<Value>) -> Self {
        let mut config = Self::default();
        let Some(val) = value else { return config };

        if let Some(app_id) = val.get("application_id").and_then(Value::as_str) {
            config.application_id = app_id.to_string();
        }
        if let Some(details) = val.get("details_format").and_then(Value::as_str) {
            config.details_format = details.to_string();
        }
        if let Some(state) = val.get("state_format").and_then(Value::as_str) {
            config.state_format = state.to_string();
        }
        if let Some(show_git) = val.get("show_git").and_then(Value::as_bool) {
            config.show_git = show_git;
        }
        if let Some(show_lines) = val.get("show_line_count").and_then(Value::as_bool) {
            config.show_line_count = show_lines;
        }
        if let Some(show_time) = val.get("show_timestamps").and_then(Value::as_bool) {
            config.show_timestamps = show_time;
        }
        if let Some(idle) = val.get("idle_timeout_seconds").and_then(Value::as_u64) {
            config.idle_timeout_seconds = idle;
        }
        if let Some(url) = val.get("base_icons_url").and_then(Value::as_str) {
            config.base_icons_url = url.to_string();
        }

        config
    }
}
