use crate::core::plugin::{ItemIcon, QueryResult};
use crate::theme::YasalTheme;
use egui::{Align2, FontId, Rect, Rounding, Sense, Stroke, Ui, Vec2};

pub enum ResultListAction {
    Select(usize),
    Execute(usize),
}

pub fn render_result_list(
    ui: &mut Ui,
    theme: &YasalTheme,
    results: &[QueryResult],
    selected_index: usize,
) -> Option<ResultListAction> {
    if results.is_empty() {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 120.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Tidak ada hasil",
            FontId::proportional(14.0),
            theme.text_secondary,
        );
        return None;
    }

    let row_height = 40.0;
    let icon_size = 24.0;
    let mut triggered_action = None;

    ui.vertical(|ui| {
        for (idx, item) in results.iter().enumerate() {
            let (row_rect, row_response) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), row_height),
                Sense::click(),
            );

            let is_selected = idx == selected_index;
            let is_hovered = row_response.hovered();

            // Background & Selection highlight
            if is_selected {
                ui.painter().rect(
                    row_rect,
                    Rounding::same(6.0),
                    theme.selection_color,
                    Stroke::NONE,
                );
                // Left accent bar (3px)
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

            // Icon container (24x24)
            let icon_rect = Rect::from_center_size(
                row_rect.left_center() + Vec2::new(26.0, 0.0),
                Vec2::splat(icon_size),
            );

            ui.painter().rect(
                icon_rect,
                Rounding::same(4.0),
                theme.surface_color,
                Stroke::new(0.5_f32, theme.border_color),
            );

            let icon_char = match &item.icon {
                ItemIcon::App(_) => "🖥️",
                ItemIcon::Symbolic(s) => match *s {
                    "calc" => "🔢",
                    "web" => "🌐",
                    "shell" => "⚡",
                    "power" => "⚡",
                    "lock" => "🔒",
                    "moon" => "🌙",
                    "trash" => "🗑️",
                    _ => "⚙️",
                },
                ItemIcon::File(_) => "📁",
            };

            ui.painter().text(
                icon_rect.center(),
                Align2::CENTER_CENTER,
                icon_char,
                FontId::proportional(14.0),
                theme.text_primary,
            );

            // Title (14px) and Subtitle (12px)
            let title_pos = row_rect.left_center() + Vec2::new(48.0, if item.subtitle.is_some() { -7.0 } else { 0.0 });
            ui.painter().text(
                title_pos,
                Align2::LEFT_CENTER,
                &item.title,
                FontId::proportional(13.5),
                theme.text_primary,
            );

            if let Some(ref subtitle) = item.subtitle {
                let sub_pos = row_rect.left_center() + Vec2::new(48.0, 8.0);
                let truncated = if subtitle.len() > 60 {
                    format!("{}...", &subtitle[..57])
                } else {
                    subtitle.clone()
                };
                ui.painter().text(
                    sub_pos,
                    Align2::LEFT_CENTER,
                    truncated,
                    FontId::proportional(11.0),
                    theme.text_secondary,
                );
            }

            // Category badge on right side
            let badge_pos = row_rect.right_center() + Vec2::new(-16.0, 0.0);
            ui.painter().text(
                badge_pos,
                Align2::RIGHT_CENTER,
                &item.category,
                FontId::proportional(11.0),
                theme.text_secondary,
            );

            if row_response.clicked() {
                triggered_action = Some(ResultListAction::Execute(idx));
            } else if is_hovered && !is_selected {
                triggered_action = Some(ResultListAction::Select(idx));
            }
        }
    });

    triggered_action
}
