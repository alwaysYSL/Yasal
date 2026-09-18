use crate::core::plugin::ItemAction;
use crate::theme::YasalTheme;
use egui::{Align2, FontId, Rect, Rounding, Sense, Stroke, Ui, Vec2};

pub enum ActionPanelEvent {
    Select(usize),
    Execute(usize),
    Close,
}

pub fn render_action_panel(
    ui: &mut Ui,
    theme: &YasalTheme,
    actions: &[ItemAction],
    selected_index: usize,
) -> Option<ActionPanelEvent> {
    if actions.is_empty() {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Tidak ada aksi tambahan untuk item ini",
            FontId::proportional(13.0),
            theme.text_secondary,
        );
        return None;
    }

    let row_height = 40.0;
    let mut triggered = None;

    ui.vertical(|ui| {
        for (idx, action) in actions.iter().enumerate() {
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

            // Action icon
            let icon_pos = row_rect.left_center() + Vec2::new(24.0, 0.0);
            ui.painter().text(
                icon_pos,
                Align2::CENTER_CENTER,
                action.icon,
                FontId::proportional(15.0),
                theme.text_primary,
            );

            // Action label
            let label_pos = row_rect.left_center() + Vec2::new(44.0, 0.0);
            ui.painter().text(
                label_pos,
                Align2::LEFT_CENTER,
                &action.label,
                FontId::proportional(13.5),
                theme.text_primary,
            );

            // Shortcut hint on right side
            if let Some(ref hint) = action.shortcut_hint {
                let hint_pos = row_rect.right_center() + Vec2::new(-16.0, 0.0);
                ui.painter().text(
                    hint_pos,
                    Align2::RIGHT_CENTER,
                    hint,
                    FontId::proportional(11.0),
                    theme.accent_color,
                );
            }

            if row_response.clicked() {
                triggered = Some(ActionPanelEvent::Execute(idx));
            } else if is_hovered && !is_selected {
                triggered = Some(ActionPanelEvent::Select(idx));
            }
        }
    });

    triggered
}
