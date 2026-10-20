use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreference {
    Auto,
    Dark,
    Light,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub hotkey: String,
    pub theme: ThemePreference,
    pub run_on_startup: bool,
    pub search_engine: String,
    pub enable_calculator: bool,
    pub enable_web_search: bool,
    pub enable_shell_command: bool,
    pub enable_system_commands: bool,
    pub enable_file_search: bool,
    pub enable_windows_settings: bool,
    pub enable_clipboard_history: bool,
    pub enable_network_info: bool,
    pub enable_window_switcher: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "Alt+Space".to_string(),
            theme: ThemePreference::Auto,
            run_on_startup: false,
            search_engine: "Google".to_string(),
            enable_calculator: true,
            enable_web_search: true,
            enable_shell_command: true,
            enable_system_commands: true,
            enable_file_search: true,
            enable_windows_settings: true,
            enable_clipboard_history: true,
            enable_network_info: true,
            enable_window_switcher: true,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Yasal")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            let default_cfg = Self::default();
            let _ = default_cfg.save();
            default_cfg
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, json)
    }
}
