use crate::theme::YasalTheme;
use egui::{Align2, FontId, Id, Key, RichText, Rounding, Stroke, Ui, Vec2};

pub struct SearchBarState {
    pub text: String,
    pub request_focus: bool,
}

impl Default for SearchBarState {
    fn default() -> Self {
        Self {
            text: String::new(),
            request_focus: true,
        }
    }
}

pub enum SearchBarAction {
    None,
    TextChanged,
    Submit,
    EscapePressed,
    NavigateUp,
    NavigateDown,
    OpenActionPanel,
    SecondaryAction,
}

pub fn render_search_bar(
    ui: &mut Ui,
    theme: &YasalTheme,
    state: &mut SearchBarState,
    mode_icon: &str,
    breadcrumb: Option<&str>,
) -> SearchBarAction {
    let mut action = SearchBarAction::None;

    // Check hotkeys directly on UI input
    if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
        action = SearchBarAction::NavigateUp;
    } else if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
        action = SearchBarAction::NavigateDown;
    } else if ui.input(|i| i.key_pressed(Key::Tab)) {
        action = SearchBarAction::OpenActionPanel;
    } else if ui.input(|i| i.key_pressed(Key::Enter) && i.modifiers.ctrl) {
        action = SearchBarAction::SecondaryAction;
    } else if ui.input(|i| i.key_pressed(Key::Enter)) {
        action = SearchBarAction::Submit;
    } else if ui.input(|i| i.key_pressed(Key::Escape)) {
        action = SearchBarAction::EscapePressed;
    }

    let search_bar_height = 46.0;
    let available_width = ui.available_width();
    let (rect, _response) = ui.allocate_exact_size(
        Vec2::new(available_width, search_bar_height),
        egui::Sense::hover(),
    );

    // Background container
    ui.painter().rect(
        rect,
        Rounding::same(8.0),
        theme.surface_color,
        Stroke::new(1.0_f32, theme.border_color),
    );

    // Left Icon
    let icon_symbol = match mode_icon {
        "calc" => "🔢",
        "shell" => "⚡",
        "web" => "🌐",
        "settings" => "⚙️",
        "action_panel" => "↩️",
        _ => "🔍",
    };

    let icon_pos = rect.left_center() + Vec2::new(16.0, 0.0);
    ui.painter().text(
        icon_pos,
        Align2::LEFT_CENTER,
        icon_symbol,
        FontId::proportional(18.0),
        theme.text_secondary,
    );

    // Text edit area
    let mut text_rect = rect;
    text_rect.min.x += 44.0;
    text_rect.max.x -= 16.0;

    let placeholder = if let Some(bc) = breadcrumb {
        format!("{} > Ketik aksi...", bc)
    } else {
        match mode_icon {
            "calc" => "Ketik ekspresi matematika (mis. 25 * 4)...".to_string(),
            "shell" => "Ketik perintah PowerShell (mis. > ipconfig)...".to_string(),
            "web" => "Ketik pencarian web atau URL...".to_string(),
            _ => "Cari aplikasi, file, kalkulator, atau perintah...".to_string(),
        }
    };

    let id = Id::new("yasal_search_input");
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));

    let edit = egui::TextEdit::singleline(&mut state.text)
        .id(id)
        .font(FontId::proportional(15.0))
        .text_color(theme.text_primary)
        .frame(false)
        .hint_text(RichText::new(placeholder).color(theme.text_secondary));

    let res = child_ui.add_sized(text_rect.size(), edit);

    if state.request_focus {
        res.request_focus();
        state.request_focus = false;
    }

    if res.changed() {
        action = SearchBarAction::TextChanged;
    }

    action
}
