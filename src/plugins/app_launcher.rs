use crate::core::matcher::Matcher;
use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use crate::platform::win32;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use windows::core::HSTRING;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{
    BHID_EnumItems, FOLDERID_AppsFolder, IEnumShellItems, IShellItem, SHGetKnownFolderItem,
    ShellExecuteW, KF_FLAG_DEFAULT, SIGDN, SIGDN_NORMALDISPLAY, SIGDN_PARENTRELATIVEPARSING,
};
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
            scan_dirs
                .push(PathBuf::from(program_data).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        if let Some(desktop) = dirs::desktop_dir() {
            scan_dirs.push(desktop);
        }

        for dir in scan_dirs {
            if dir.exists() {
                self.scan_directory(&dir, &mut entries);
            }
        }

        // AppsFolder includes packaged UWP/MSIX apps as well as some Win32 apps.
        // Scan it after shortcuts so name collisions retain the existing entry.
        self.scan_apps_folder(&mut entries);

        let mut seen_names = HashSet::new();
        entries.retain(|entry| seen_names.insert(entry.name.to_lowercase()));
        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

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

    fn scan_apps_folder(&self, out: &mut Vec<AppEntry>) {
        unsafe {
            let folder: IShellItem =
                match SHGetKnownFolderItem(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, None) {
                    Ok(folder) => folder,
                    Err(_) => return,
                };
            let items: IEnumShellItems = match folder.BindToHandler(None, &BHID_EnumItems) {
                Ok(items) => items,
                Err(_) => return,
            };

            loop {
                let mut batch = [None];
                let mut fetched = 0;
                if items.Next(&mut batch, Some(&mut fetched)).is_err() || fetched == 0 {
                    break;
                }

                let Some(item) = batch[0].take() else {
                    continue;
                };
                let Some(name) = shell_item_display_name(&item, SIGDN_NORMALDISPLAY) else {
                    continue;
                };
                let Some(app_id) = shell_item_display_name(&item, SIGDN_PARENTRELATIVEPARSING)
                else {
                    continue;
                };

                if !name.is_empty() && !app_id.is_empty() {
                    let launch_target = if is_shell_apps_folder_target(&app_id) {
                        app_id
                    } else {
                        format!(r"shell:AppsFolder\{app_id}")
                    };
                    out.push(AppEntry {
                        id: format!("app:{}", name.to_lowercase()),
                        name,
                        path: PathBuf::from(launch_target),
                    });
                }
            }
        }
    }
}

unsafe fn shell_item_display_name(item: &IShellItem, format: SIGDN) -> Option<String> {
    let value = item.GetDisplayName(format).ok()?;
    let result = value.to_string().ok();
    CoTaskMemFree(Some(value.0.cast()));
    result
}

fn is_shell_apps_folder_target(target: &str) -> bool {
    target
        .get(..17)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(r"shell:AppsFolder\"))
}

fn shell_execute(operation: &str, target: &str) -> bool {
    let operation = HSTRING::from(operation);
    let target = HSTRING::from(target);
    unsafe { ShellExecuteW(None, &operation, &target, None, None, SW_SHOWNORMAL).0 as isize > 32 }
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
        let mut matches: Vec<(&AppEntry, u32)> = self
            .apps
            .iter()
            .filter_map(|app| {
                let score = if input.is_empty() {
                    Some(10)
                } else {
                    self.matcher.fuzzy_score(input, &app.name)
                };
                score.map(|score| (app, score))
            })
            .collect();
        matches.sort_unstable_by(|(_, left), (_, right)| right.cmp(left));
        matches.truncate(8);

        let mut results = Vec::with_capacity(matches.len());
        for (app, score) in matches {
            let target = app.path.to_string_lossy().to_string();
            let is_shell_app = is_shell_apps_folder_target(&target);
            let path_for_launch = app.path.clone();
            let target_for_launch = target.clone();

            let primary_action: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                let opened = if is_shell_app {
                    shell_execute("open", &target_for_launch)
                } else {
                    open::that_detached(&path_for_launch).is_ok()
                };
                if !opened {
                    win32::show_error_dialog(
                        "Aplikasi Tidak Dapat Dibuka",
                        "Target aplikasi tidak tersedia atau shortcut sudah tidak valid.",
                    );
                }
            });

            let (secondary_action, additional_actions) = if is_shell_app {
                let target_for_copy = target.clone();
                let apps_folder_action = ItemAction {
                    label: "Buka daftar aplikasi Windows".to_string(),
                    icon: "📁",
                    shortcut_hint: None,
                    action: Arc::new(|| {
                        if !shell_execute("open", "shell:AppsFolder") {
                            win32::show_error_dialog(
                                "AppsFolder Tidak Dapat Dibuka",
                                "Windows tidak dapat membuka daftar aplikasi.",
                            );
                        }
                    }),
                };
                let copy_id_action = ItemAction {
                    label: "Salin ID aplikasi".to_string(),
                    icon: "📋",
                    shortcut_hint: None,
                    action: Arc::new(move || {
                        let _ = set_clipboard_text(&target_for_copy);
                    }),
                };
                (None, vec![apps_folder_action, copy_id_action])
            } else {
                let path_for_admin = app.path.clone();
                let path_for_folder = app.path.clone();
                let path_for_copy = app.path.clone();
                let admin_action = ItemAction {
                    label: "Jalankan sebagai administrator".to_string(),
                    icon: "🛡️",
                    shortcut_hint: Some("Ctrl+Enter".to_string()),
                    action: Arc::new(move || {
                        if !shell_execute("runas", &path_for_admin.to_string_lossy()) {
                            win32::show_error_dialog(
                                "Aplikasi Tidak Dapat Dijalankan",
                                "Windows menolak atau gagal menjalankan aplikasi sebagai administrator.",
                            );
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
                (
                    Some(admin_action.clone()),
                    vec![admin_action, folder_action, copy_path_action],
                )
            };

            results.push(QueryResult {
                id: app.id.clone(),
                title: app.name.clone(),
                subtitle: Some(target.clone()),
                category: "Aplikasi".to_string(),
                icon: ItemIcon::App(target),
                score,
                primary_action,
                secondary_action,
                additional_actions,
            });
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
                    std::ptr::copy_nonoverlapping(
                        utf16.as_ptr() as *const u8,
                        ptr as *mut u8,
                        bytes,
                    );
                    let _ = GlobalUnlock(handle);
                    let _ = SetClipboardData(13, HANDLE(handle.0)); // 13 = CF_UNICODETEXT
                }
            }
            let _ = CloseClipboard();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_apps_folder_entries_use_shell_actions() {
        let target = r"shell:AppsFolder\Contoso.App_123!App";
        let launcher =
            AppLauncherPlugin::new_with_mock(vec![("Contoso App".to_string(), target.to_string())]);

        let result = launcher.query("contoso").remove(0);

        assert_eq!(result.subtitle.as_deref(), Some(target));
        assert!(result.secondary_action.is_none());
        assert_eq!(result.additional_actions.len(), 2);
    }

    #[test]
    fn shell_apps_folder_target_detection_is_case_insensitive() {
        assert!(is_shell_apps_folder_target(
            r"SHELL:APPSFOLDER\Contoso.App_123!App"
        ));
        assert!(!is_shell_apps_folder_target(r"C:\Apps\Contoso.exe"));
    }
}
