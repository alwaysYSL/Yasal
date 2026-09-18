use crate::config::{AppConfig, ThemePreference};
use crate::theme::YasalTheme;
use egui::{Align2, FontId, Rect, Rounding, Sense, Stroke, Ui, Vec2};

pub enum SettingsViewEvent {
    Select(usize),
    Toggle(usize),
    Close,
}

pub struct SettingItem {
    pub id: &'static str,
    pub title: &'static str,
    pub value_display: String,
    pub icon: &'static str,
}

pub fn get_setting_items(config: &AppConfig) -> Vec<SettingItem> {
    vec![
        SettingItem {
            id: "hotkey",
            title: "Global Hotkey",
            value_display: config.hotkey.clone(),
            icon: "⌨️",
        },
        SettingItem {
            id: "startup",
            title: "Jalankan saat Windows Startup",
            value_display: if config.run_on_startup { "[ON]".to_string() } else { "[OFF]".to_string() },
            icon: "🚀",
        },
        SettingItem {
            id: "theme",
            title: "Tema Tampilan",
            value_display: match config.theme {
                ThemePreference::Auto => "Auto (Sistem)".to_string(),
                ThemePreference::Dark => "Gelap".to_string(),
                ThemePreference::Light => "Terang".to_string(),
            },
            icon: "🎨",
        },
        SettingItem {
            id: "calc",
            title: "Plugin: Kalkulator",
            value_display: if config.enable_calculator { "[ON]".to_string() } else { "[OFF]".to_string() },
            icon: "🔢",
        },
        SettingItem {
            id: "web",
            title: "Plugin: Pencarian Web",
            value_display: if config.enable_web_search { "[ON]".to_string() } else { "[OFF]".to_string() },
            icon: "🌐",
        },
        SettingItem {
            id: "shell",
            title: "Plugin: Perintah Shell",
            value_display: if config.enable_shell_command { "[ON]".to_string() } else { "[OFF]".to_string() },
            icon: "⚡",
        },
        SettingItem {
            id: "sys",
            title: "Plugin: Perintah Sistem",
            value_display: if config.enable_system_commands { "[ON]".to_string() } else { "[OFF]".to_string() },
            icon: "🛡️",
        },
        SettingItem {
            id: "engine",
            title: "Search Engine Default",
            value_display: config.search_engine.clone(),
            icon: "🔍",
        },
        SettingItem {
            id: "about",
            title: "Tentang Yasal",
            value_display: "v0.1.0 · Ultra-Ringan".to_string(),
            icon: "🌸",
        },
    ]
}

pub fn render_settings_view(
    ui: &mut Ui,
    theme: &YasalTheme,
    config: &mut AppConfig,
    filter: &str,
    selected_index: usize,
) -> Option<SettingsViewEvent> {
    let all_items = get_setting_items(config);
    let items: Vec<SettingItem> = if filter.is_empty() {
        all_items
    } else {
        all_items
            .into_iter()
            .filter(|i| i.title.to_lowercase().contains(&filter.to_lowercase()))
            .collect()
    };

    if items.is_empty() {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Tidak ada pengaturan yang cocok",
            FontId::proportional(13.0),
            theme.text_secondary,
        );
        return None;
    }

    let row_height = 40.0;
    let mut triggered = None;

    ui.vertical(|ui| {
        for (idx, item) in items.iter().enumerate() {
            let (row_rect, row_response) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), row_height),
                Sense::click(),
            );

            let is_selected = idx == selected_index;
            let is_hovered = row_response.hovered();

            if is_selected {
                ui.painter().rect(
                    row_rect,
                    Rounding::same(6.0),
                    theme.selection_color,
                    Stroke::NONE,
                );
                let accent_bar = Rect::from_min_size(
                    row_rect.left_top() + Vec2::new(2.0, 4.0),
                    Vec2::new(3.5, row_height - 8.0),
                );
                ui.painter().rect(
                    accent_bar,
                    Rounding::same(2.0),
                    theme.accent_color,
                    Stroke::NONE,
                );
            } else if is_hovered {
                ui.painter().rect(
                    row_rect,
                    Rounding::same(6.0),
                    theme.hover_color,
                    Stroke::NONE,
                );
            }

            // Setting Icon
            let icon_pos = row_rect.left_center() + Vec2::new(24.0, 0.0);
            ui.painter().text(
                icon_pos,
                Align2::CENTER_CENTER,
                item.icon,
                FontId::proportional(14.0),
                theme.text_primary,
            );

            // Setting Title
            let title_pos = row_rect.left_center() + Vec2::new(44.0, 0.0);
            ui.painter().text(
                title_pos,
                Align2::LEFT_CENTER,
                item.title,
                FontId::proportional(13.5),
                theme.text_primary,
            );

            // Setting Value Badge on right side
            let val_pos = row_rect.right_center() + Vec2::new(-16.0, 0.0);
            ui.painter().text(
                val_pos,
                Align2::RIGHT_CENTER,
                &item.value_display,
                FontId::proportional(12.0),
                if is_selected { theme.accent_color } else { theme.text_secondary },
            );

            if row_response.clicked() {
                triggered = Some(SettingsViewEvent::Toggle(idx));
            } else if is_hovered && !is_selected {
                triggered = Some(SettingsViewEvent::Select(idx));
            }
        }
    });

    triggered
}

pub fn toggle_setting_at(config: &mut AppConfig, idx: usize, filter: &str) {
    let items = get_setting_items(config);
    let filtered_ids: Vec<&str> = if filter.is_empty() {
        items.iter().map(|i| i.id).collect()
    } else {
        items
            .iter()
            .filter(|i| i.title.to_lowercase().contains(&filter.to_lowercase()))
            .map(|i| i.id)
            .collect()
    };

    if let Some(&id) = filtered_ids.get(idx) {
        match id {
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
            "engine" => {
                config.search_engine = match config.search_engine.as_str() {
                    "Google" => "DuckDuckGo".to_string(),
                    "DuckDuckGo" => "Bing".to_string(),
                    _ => "Google".to_string(),
                };
            }
            _ => {}
        }
        config.save();
    }
}
