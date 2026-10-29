use crate::config::{AppConfig, ThemePreference, MAX_SEARCH_FOLDERS};
use crate::theme::{YasalTheme, ROW_CORNER_RADIUS, ROW_HEIGHT};
use crate::ui::icons::paint_icon;
use egui::{Align, Align2, FontId, Id, Rounding, Sense, Stroke, Ui, Vec2};

pub enum SettingsViewEvent {
    Select(usize),
    Toggle(usize),
    Close,
}

pub struct SettingItem {
    pub id: String,
    pub title: String,
    pub value_display: String,
    pub icon: &'static str,
    pub interactive: bool,
}

/// Returns the filter following the `settings` command, if this is a settings query.
pub fn settings_query_filter(query: &str) -> Option<&str> {
    let trimmed = query.trim();
    let command = trimmed.get(..8)?;
    if !command.eq_ignore_ascii_case("settings") {
        return None;
    }

    let remainder = trimmed.get(8..).unwrap_or_default();
    if remainder.is_empty() || remainder.starts_with(char::is_whitespace) {
        Some(remainder.trim())
    } else {
        None
    }
}

pub fn settings_filter(query: &str) -> &str {
    settings_query_filter(query).unwrap_or_else(|| query.trim())
}

pub fn filtered_setting_count(config: &AppConfig, query: &str) -> usize {
    let filter = settings_filter(query).to_lowercase();
    get_setting_items(config)
        .iter()
        .filter(|item| {
            filter.is_empty()
                || item.title.to_lowercase().contains(&filter)
                || item.id.contains(&filter)
        })
        .count()
}

/// Shortens a path to at most `max_chars` characters, keeping its end, so the
/// folder name stays visible in a fixed-width row.
fn shorten_path(path: &str, max_chars: usize) -> String {
    let count = path.chars().count();
    if count <= max_chars {
        return path.to_string();
    }
    let tail: String = path.chars().skip(count - (max_chars - 1)).collect();
    format!("…{tail}")
}

pub fn get_setting_items(config: &AppConfig) -> Vec<SettingItem> {
    let mut items = vec![
        SettingItem {
            id: "hotkey".into(),
            title: "Pintasan Global".into(),
            value_display: config.hotkey.clone(),
            icon: "hotkey",
            interactive: true,
        },
        SettingItem {
            id: "startup".into(),
            title: "Mulai bersama Windows".into(),
            value_display: if config.run_on_startup {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "startup",
            interactive: true,
        },
        SettingItem {
            id: "theme".into(),
            title: "Tema Tampilan".into(),
            value_display: match config.theme {
                ThemePreference::Auto => "Otomatis (Sistem)".to_string(),
                ThemePreference::Dark => "Gelap".to_string(),
                ThemePreference::Light => "Terang".to_string(),
            },
            icon: "theme",
            interactive: true,
        },
        SettingItem {
            id: "calc".into(),
            title: "Plugin: Kalkulator".into(),
            value_display: if config.enable_calculator {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "calc",
            interactive: true,
        },
        SettingItem {
            id: "web".into(),
            title: "Plugin: Pencarian Web".into(),
            value_display: if config.enable_web_search {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "web",
            interactive: true,
        },
        SettingItem {
            id: "shell".into(),
            title: "Plugin: Perintah Shell".into(),
            value_display: if config.enable_shell_command {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "shell",
            interactive: true,
        },
        SettingItem {
            id: "sys".into(),
            title: "Plugin: Perintah Sistem".into(),
            value_display: if config.enable_system_commands {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "sys",
            interactive: true,
        },
        SettingItem {
            id: "file".into(),
            title: "Plugin: Pencarian File".into(),
            value_display: if config.enable_file_search {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "file",
            interactive: true,
        },
        SettingItem {
            id: "win_settings".into(),
            title: "Plugin: Pengaturan Windows".into(),
            value_display: if config.enable_windows_settings {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "settings",
            interactive: true,
        },
        SettingItem {
            id: "clipboard".into(),
            title: "Plugin: Riwayat Clipboard".into(),
            value_display: if config.enable_clipboard_history {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "clipboard",
            interactive: true,
        },
        SettingItem {
            id: "network".into(),
            title: "Plugin: Informasi Jaringan".into(),
            value_display: if config.enable_network_info {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "network",
            interactive: true,
        },
        SettingItem {
            id: "windows".into(),
            title: "Plugin: Pengalih Jendela".into(),
            value_display: if config.enable_window_switcher {
                "[AKTIF]".to_string()
            } else {
                "[NONAKTIF]".to_string()
            },
            icon: "window",
            interactive: true,
        },
        SettingItem {
            id: "engine".into(),
            title: "Mesin Pencari Bawaan".into(),
            value_display: config.search_engine.clone(),
            icon: "search",
            interactive: true,
        },
    ];

    // Folder scope for file search. The header row adds a folder; each folder row below it
    // removes that folder when activated.
    let folder_value = if config.file_search_folders.is_empty() {
        "Default".to_string()
    } else {
        format!(
            "{} folder",
            config.file_search_folders.len().min(MAX_SEARCH_FOLDERS)
        )
    };
    items.push(SettingItem {
        id: "folder_add".into(),
        title: "Folder Pencarian File".into(),
        value_display: folder_value,
        icon: "folder",
        interactive: true,
    });
    for (index, folder) in config.file_search_folders.iter().enumerate() {
        items.push(SettingItem {
            id: format!("folder:{index}"),
            title: format!("· {}", shorten_path(folder, 56)),
            value_display: "[HAPUS]".to_string(),
            icon: "folder",
            interactive: true,
        });
    }

    items.push(SettingItem {
        id: "about".into(),
        title: "Tentang Yasal".into(),
        value_display: "v0.1.0 · Sangat ringan".to_string(),
        icon: "about",
        interactive: false,
    });
    items
}

pub fn render_settings_view(
    ui: &mut Ui,
    theme: &YasalTheme,
    config: &mut AppConfig,
    filter: &str,
    selected_index: usize,
) -> Option<SettingsViewEvent> {
    let all_items = get_setting_items(config);
    let filter = settings_filter(filter).to_lowercase();
    let items: Vec<SettingItem> = if filter.is_empty() {
        all_items
    } else {
        all_items
            .into_iter()
            .filter(|item| item.title.to_lowercase().contains(&filter) || item.id.contains(&filter))
            .collect()
    };

    if items.is_empty() {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Tidak ada pengaturan yang cocok",
            FontId::proportional(13.0),
            theme.text_secondary,
        );
        return None;
    }

    let row_height = ROW_HEIGHT;
    let mut triggered = None;

    // Scroll only when the selection or filter changed (see result_list.rs).
    let scroll_state_id = Id::new("yasal_settings_last_scrolled");
    let scroll_key = (selected_index, filter.clone());
    let should_scroll = ui
        .ctx()
        .data(|data| data.get_temp::<(usize, String)>(scroll_state_id))
        .as_ref()
        != Some(&scroll_key);

    ui.vertical(|ui| {
        for (idx, item) in items.iter().enumerate() {
            let sense = if item.interactive {
                Sense::click()
            } else {
                Sense::hover()
            };
            let (row_rect, row_response) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), row_height), sense);

            let is_selected = idx == selected_index;
            let is_hovered = row_response.hovered();

            if is_selected && should_scroll {
                ui.scroll_to_rect(row_rect, Some(Align::Center));
            }

            if is_selected {
                ui.painter().rect(
                    row_rect,
                    Rounding::same(ROW_CORNER_RADIUS),
                    theme.selection_color,
                    Stroke::NONE,
                );
            } else if is_hovered {
                ui.painter().rect(
                    row_rect,
                    Rounding::same(ROW_CORNER_RADIUS),
                    theme.hover_color,
                    Stroke::NONE,
                );
            }

            // Setting Icon
            let icon_pos = row_rect.left_center() + Vec2::new(28.0, 0.0);
            paint_icon(
                ui.painter(),
                icon_pos,
                item.icon,
                17.0,
                if item.interactive {
                    theme.text_primary
                } else {
                    theme.text_secondary
                },
            );

            // Setting Title
            let title_pos = row_rect.left_center() + Vec2::new(52.0, 0.0);
            ui.painter().text(
                title_pos,
                Align2::LEFT_CENTER,
                &item.title,
                FontId::proportional(14.0),
                if item.interactive {
                    theme.text_primary
                } else {
                    theme.text_secondary
                },
            );

            // Setting Value Badge on right side
            let val_pos = row_rect.right_center() + Vec2::new(-16.0, 0.0);
            ui.painter().text(
                val_pos,
                Align2::RIGHT_CENTER,
                &item.value_display,
                FontId::proportional(12.0),
                if is_selected && item.interactive {
                    theme.accent_text
                } else {
                    theme.text_secondary
                },
            );

            if item.interactive && row_response.clicked() {
                triggered = Some(SettingsViewEvent::Toggle(idx));
            } else if is_hovered && !is_selected && ui.input(|i| i.pointer.delta() != Vec2::ZERO) {
                // Follow real pointer movement only, so a resting cursor does not
                // override keyboard navigation.
                triggered = Some(SettingsViewEvent::Select(idx));
            }
        }
    });

    if should_scroll {
        ui.ctx()
            .data_mut(|data| data.insert_temp(scroll_state_id, scroll_key));
    }

    triggered
}

/// Hotkeys offered by the Settings view, cycled with Enter.
pub const HOTKEY_PRESETS: &[&str] = &[
    "Alt+Space",
    "Ctrl+Space",
    "Ctrl+Shift+Space",
    "Ctrl+Alt+Space",
    "Alt+Shift+Space",
    "Ctrl+Shift+Y",
];

/// The preset after `current` (wrapping). An unknown value starts the cycle at the first.
pub fn next_hotkey_preset(current: &str) -> &'static str {
    let position = HOTKEY_PRESETS
        .iter()
        .position(|preset| preset.eq_ignore_ascii_case(current));
    HOTKEY_PRESETS[position.map_or(0, |index| (index + 1) % HOTKEY_PRESETS.len())]
}

/// Ids of rows that change a value directly. Used to return a `'static` id from a
/// runtime-built row list.
const TOGGLE_IDS: &[&str] = &[
    "hotkey",
    "startup",
    "theme",
    "calc",
    "web",
    "shell",
    "sys",
    "file",
    "win_settings",
    "clipboard",
    "network",
    "windows",
    "engine",
    "folder_add",
];

/// Applies the row at `idx` (within the current filter) and returns the setting id that
/// changed, so the caller can apply it. `folder_add` is returned without a change: the
/// caller opens the folder picker. Folder rows remove their folder and return `file`.
pub fn toggle_setting_at(config: &mut AppConfig, idx: usize, filter: &str) -> Option<&'static str> {
    let items = get_setting_items(config);
    let filter = settings_filter(filter).to_lowercase();
    let filtered_ids: Vec<String> = if filter.is_empty() {
        items.iter().map(|i| i.id.clone()).collect()
    } else {
        items
            .iter()
            .filter(|item| item.title.to_lowercase().contains(&filter) || item.id.contains(&filter))
            .map(|i| i.id.clone())
            .collect()
    };

    let id = filtered_ids.get(idx)?.as_str();
    if let Some(folder_index) = id
        .strip_prefix("folder:")
        .and_then(|index| index.parse::<usize>().ok())
    {
        if folder_index < config.file_search_folders.len() {
            config.file_search_folders.remove(folder_index);
        }
        return Some("file");
    }

    let known: &'static str = TOGGLE_IDS.iter().copied().find(|known| *known == id)?;
    match known {
        "hotkey" => config.hotkey = next_hotkey_preset(&config.hotkey).to_string(),
        "startup" => config.run_on_startup = !config.run_on_startup,
        "theme" => {
            config.theme = match config.theme {
                ThemePreference::Auto => ThemePreference::Dark,
                ThemePreference::Dark => ThemePreference::Light,
                ThemePreference::Light => ThemePreference::Auto,
            };
        }
        "calc" => config.enable_calculator = !config.enable_calculator,
        "web" => config.enable_web_search = !config.enable_web_search,
        "shell" => config.enable_shell_command = !config.enable_shell_command,
        "sys" => config.enable_system_commands = !config.enable_system_commands,
        "file" => config.enable_file_search = !config.enable_file_search,
        "win_settings" => config.enable_windows_settings = !config.enable_windows_settings,
        "clipboard" => config.enable_clipboard_history = !config.enable_clipboard_history,
        "network" => config.enable_network_info = !config.enable_network_info,
        "windows" => config.enable_window_switcher = !config.enable_window_switcher,
        "engine" => {
            config.search_engine = match config.search_engine.as_str() {
                "Google" => "DuckDuckGo".to_string(),
                "DuckDuckGo" => "Bing".to_string(),
                _ => "Google".to_string(),
            };
        }
        "folder_add" => {}
        _ => return None,
    }
    Some(known)
}
