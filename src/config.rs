use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Most folders the user can add for file search. Everything runs one query per folder.
pub const MAX_SEARCH_FOLDERS: usize = 8;

/// Keywords that the palette already uses as command prefixes. An alias with one of these
/// keywords would never run, because the prefix is matched first.
const RESERVED_ALIAS_KEYWORDS: &[&str] = &["file", "cb", "ip", "network", "w", "settings"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreference {
    Auto,
    Dark,
    Light,
}

/// A keyword that opens a web search on one site, for example `yt rust tutorial`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebAlias {
    /// Typed before the search text. Only letters, digits, `-` and `_`, up to 16 characters.
    pub keyword: String,
    /// Address containing `{query}`. The placeholder is replaced with the URL-encoded search text.
    pub url: String,
}

impl WebAlias {
    /// Whether the alias can be used. Invalid aliases are skipped instead of failing the app.
    pub fn is_valid(&self) -> bool {
        let keyword = self.keyword.as_str();
        let keyword_ok = !keyword.is_empty()
            && keyword.len() <= 16
            && keyword
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            && !RESERVED_ALIAS_KEYWORDS
                .iter()
                .any(|reserved| reserved.eq_ignore_ascii_case(keyword));
        let url_ok = (self.url.starts_with("https://") || self.url.starts_with("http://"))
            && self.url.contains("{query}");
        keyword_ok && url_ok
    }
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
    /// Folders that file search is limited to. Empty means the default scope.
    pub file_search_folders: Vec<String>,
    /// Keyword shortcuts for web search, edited in config.json.
    pub web_aliases: Vec<WebAlias>,
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
            file_search_folders: Vec::new(),
            web_aliases: Vec::new(),
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

    /// Folders for file search, limited to `MAX_SEARCH_FOLDERS`.
    pub fn search_folder_paths(&self) -> Vec<PathBuf> {
        self.file_search_folders
            .iter()
            .take(MAX_SEARCH_FOLDERS)
            .map(PathBuf::from)
            .collect()
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
