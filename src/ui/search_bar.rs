use crate::theme::{YasalTheme, SEARCH_BAR_HEIGHT, SEARCH_BAR_RADIUS};
use crate::ui::icons::paint_icon;
use egui::{Align, Align2, FontId, Id, Key, RichText, Rounding, Stroke, Ui, Vec2};

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

    let is_action_query = breadcrumb.is_some();
    let mut action_query = if is_action_query {
        ui.ctx().data(|data| {
            data.get_temp::<String>(Id::new("yasal_action_filter"))
                .unwrap_or_default()
        })
    } else {
        String::new()
    };

    // Check hotkeys directly on UI input
    if is_action_query && action_query.is_empty() && ui.input(|i| i.key_pressed(Key::Backspace)) {
        action = SearchBarAction::EscapePressed;
    } else if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
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

    let search_bar_height = SEARCH_BAR_HEIGHT;
    let available_width = ui.available_width();
    let (rect, _response) = ui.allocate_exact_size(
        Vec2::new(available_width, search_bar_height),
        egui::Sense::hover(),
    );

    // Background container
    ui.painter().rect(
        rect,
        Rounding::same(SEARCH_BAR_RADIUS),
        theme.surface_color,
        Stroke::new(1.0_f32, theme.border_color),
    );

    // Clear (X) button, only while there is text to clear.
    let has_text = if is_action_query {
        !action_query.is_empty()
    } else {
        !state.text.is_empty()
    };
    let mut clear_clicked = false;
    if has_text {
        let clear_rect = egui::Rect::from_center_size(
            rect.right_center() - Vec2::new(26.0, 0.0),
            Vec2::splat(24.0),
        );
        let clear_response = ui.interact(
            clear_rect,
            Id::new("yasal_search_clear"),
            egui::Sense::click(),
        );
        let color = if clear_response.hovered() {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        let c = clear_rect.center();
        let stroke = Stroke::new(1.8_f32, color);
        ui.painter()
            .line_segment([c + Vec2::new(-5.0, -5.0), c + Vec2::new(5.0, 5.0)], stroke);
        ui.painter()
            .line_segment([c + Vec2::new(-5.0, 5.0), c + Vec2::new(5.0, -5.0)], stroke);
        clear_clicked = clear_response.clicked();
    }

    // Left Icon
    let icon_name = match mode_icon {
        "calc" | "shell" | "web" | "settings" | "file" | "clipboard" | "network" | "window"
        | "action_panel" => mode_icon,
        _ => "search",
    };

    paint_icon(
        ui.painter(),
        rect.left_center() + Vec2::new(28.0, 0.0),
        icon_name,
        18.0,
        theme.text_secondary,
    );

    // Keep the breadcrumb separate from the editable action query.
    let mut text_rect = rect;
    text_rect.min.x += 44.0;
    text_rect.max.x -= if has_text { 48.0 } else { 16.0 };

    if let Some(label) = breadcrumb {
        let breadcrumb_galley = ui.painter().layout_no_wrap(
            label.to_owned(),
            FontId::proportional(13.0),
            theme.text_secondary,
        );
        let breadcrumb_width = breadcrumb_galley.size().x.min(text_rect.width() * 0.45);
        let breadcrumb_rect = egui::Rect::from_min_max(
            text_rect.min,
            egui::pos2(text_rect.min.x + breadcrumb_width, text_rect.max.y),
        );
        ui.painter().with_clip_rect(breadcrumb_rect).galley(
            egui::pos2(
                breadcrumb_rect.left(),
                breadcrumb_rect.center().y - breadcrumb_galley.size().y / 2.0,
            ),
            breadcrumb_galley,
            theme.text_secondary,
        );

        let separator_pos = breadcrumb_rect.right_center() + Vec2::new(8.0, 0.0);
        ui.painter().text(
            separator_pos,
            Align2::CENTER_CENTER,
            "›",
            FontId::proportional(15.0),
            theme.text_secondary,
        );
        text_rect.min.x = separator_pos.x + 12.0;
    }

    let placeholder = if is_action_query {
        "Cari aksi...".to_string()
    } else {
        match mode_icon {
            "calc" => "Ketik ekspresi matematika (mis. 25 * 4)...".to_string(),
            "shell" => "Ketik perintah PowerShell (mis. > ipconfig)...".to_string(),
            "web" => "Ketik pencarian web atau URL...".to_string(),
            "file" => "Cari file dan folder...".to_string(),
            "clipboard" => "Cari riwayat clipboard...".to_string(),
            "network" => "Informasi jaringan lokal...".to_string(),
            "window" => "Cari jendela yang sedang terbuka...".to_string(),
            "settings" => "Cari halaman Pengaturan Windows...".to_string(),
            _ => "Cari aplikasi, file, kalkulator, atau perintah...".to_string(),
        }
    };

    let id = Id::new("yasal_search_input");
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));

    let edit_text = if is_action_query {
        &mut action_query
    } else {
        &mut state.text
    };
    let edit = egui::TextEdit::singleline(edit_text)
        .id(id)
        .font(FontId::proportional(15.0))
        .text_color(theme.text_primary)
        .vertical_align(Align::Center)
        .frame(false)
        .hint_text(RichText::new(placeholder).color(theme.text_secondary));

    let res = child_ui.add_sized(text_rect.size(), edit);

    if state.request_focus {
        res.request_focus();
        state.request_focus = false;
    }

    if is_action_query {
        ui.ctx().data_mut(|data| {
            data.insert_temp(Id::new("yasal_action_filter"), action_query);
        });
    } else {
        ui.ctx().data_mut(|data| {
            data.remove::<String>(Id::new("yasal_action_filter"));
        });
    }

    if res.changed() {
        action = SearchBarAction::TextChanged;
    }

    if clear_clicked {
        if is_action_query {
            ui.ctx().data_mut(|data| {
                data.insert_temp(Id::new("yasal_action_filter"), String::new());
            });
        } else {
            state.text.clear();
        }
        state.request_focus = true;
        action = SearchBarAction::TextChanged;
    }

    action
}
