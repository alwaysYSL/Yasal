use crate::core::matcher::Matcher;
use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use windows::core::HSTRING;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}

pub struct AppLauncherPlugin {
    apps: Vec<AppEntry>,
    matcher: Matcher,
}

impl AppLauncherPlugin {
    pub fn new() -> Self {
        let mut plugin = Self {
            apps: Vec::new(),
            matcher: Matcher::new(),
        };
        plugin.refresh_index();
        plugin
    }

    pub fn new_with_mock(apps: Vec<(String, String)>) -> Self {
        let entries = apps
            .into_iter()
            .map(|(name, path_str)| AppEntry {
                id: format!("app:{}", name.to_lowercase()),
                name,
                path: PathBuf::from(path_str),
            })
            .collect();

        Self {
            apps: entries,
            matcher: Matcher::new(),
        }
    }

    /// Scans standard Windows Start Menu and Desktop shortcuts.
    pub fn refresh_index(&mut self) {
        let mut entries = Vec::new();

        let mut scan_dirs = Vec::new();
        if let Some(app_data) = dirs::data_dir() {
            scan_dirs.push(app_data.join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        if let Ok(program_data) = std::env::var("ALLUSERSPROFILE") {
            scan_dirs.push(PathBuf::from(program_data).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        if let Some(desktop) = dirs::desktop_dir() {
            scan_dirs.push(desktop);
        }

        for dir in scan_dirs {
            if dir.exists() {
                self.scan_directory(&dir, &mut entries);
            }
        }

        // Deduplicate entries by name
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));

        self.apps = entries;
    }

    fn scan_directory(&self, dir: &Path, out: &mut Vec<AppEntry>) {
        if let Ok(read_dir) = fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_directory(&path, out);
                } else if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if ext_str.eq_ignore_ascii_case("lnk") || ext_str.eq_ignore_ascii_case("url") {
                        let file_stem = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();

                        if !file_stem.is_empty() {
                            out.push(AppEntry {
                                id: format!("app:{}", file_stem.to_lowercase()),
                                name: file_stem,
                                path: path.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}

impl Default for AppLauncherPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AppLauncherPlugin {
    fn id(&self) -> &str {
        "app_launcher"
    }

    fn name(&self) -> &str {
        "Applications"
    }

    fn is_global(&self) -> bool {
        true
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        let mut results = Vec::new();

        for app in &self.apps {
            let score = if input.is_empty() {
                Some(10)
            } else {
                self.matcher.fuzzy_score(input, &app.name)
            };

            if let Some(score) = score {
                let path_clone = app.path.clone();
                let path_for_admin = app.path.clone();
                let path_for_folder = app.path.clone();
                let path_for_copy = app.path.clone();

                let primary_action: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                    let _ = open::that_detached(&path_clone);
                });

                let secondary_action = ItemAction {
                    label: "Run as Administrator".to_string(),
                    icon: "🛡️",
                    shortcut_hint: Some("Ctrl+Enter".to_string()),
                    action: Arc::new(move || {
                        let path_str = path_for_admin.to_string_lossy();
                        let file_w = HSTRING::from(path_str.as_ref());
                        let op_w = HSTRING::from("runas");
                        unsafe {
                            let _ = ShellExecuteW(None, &op_w, &file_w, None, None, SW_SHOWNORMAL);
                        }
                    }),
                };

                let folder_action = ItemAction {
                    label: "Buka Lokasi Folder".to_string(),
                    icon: "📁",
                    shortcut_hint: None,
                    action: Arc::new(move || {
                        if let Some(parent) = path_for_folder.parent() {
                            let _ = open::that_detached(parent);
                        }
                    }),
                };

                let copy_path_action = ItemAction {
                    label: "Salin Path File".to_string(),
                    icon: "📋",
                    shortcut_hint: None,
                    action: Arc::new(move || {
                        let path_str = path_for_copy.to_string_lossy().to_string();
                        let _ = set_clipboard_text(&path_str);
                    }),
                };

                results.push(QueryResult {
                    id: app.id.clone(),
                    title: app.name.clone(),
                    subtitle: Some(app.path.to_string_lossy().to_string()),
                    category: "Aplikasi".to_string(),
                    icon: ItemIcon::App(app.path.to_string_lossy().to_string()),
                    score,
                    primary_action,
                    secondary_action: Some(secondary_action.clone()),
                    additional_actions: vec![secondary_action, folder_action, copy_path_action],
                });
            }
        }

        results
    }
}

pub fn set_clipboard_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = utf16.len() * 2;

    unsafe {
        if OpenClipboard(None).is_ok() {
            let _ = EmptyClipboard();
            let handle = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if let Ok(handle) = handle {
                let ptr = GlobalLock(handle);
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, ptr as *mut u8, bytes);
                    let _ = GlobalUnlock(handle);
                    let _ = SetClipboardData(13, HANDLE(handle.0)); // 13 = CF_UNICODETEXT
                }
            }
            let _ = CloseClipboard();
        }
    }
    Ok(())
}
