use crate::core::plugin::{ItemIcon, QueryResult};
use crate::plugins::icon_cache::IconCache;
use crate::theme::{YasalTheme, ICON_CORNER_RADIUS, ROW_CORNER_RADIUS, ROW_HEIGHT};
use crate::ui::icons::paint_icon;
use egui::text::LayoutJob;
use egui::{Align, Align2, Color32, FontId, Id, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use std::sync::Arc;

pub enum ResultListAction {
    Select(usize),
    Execute(usize),
}

const ICON_SIZE: f32 = 28.0;
const TEXT_LEFT: f32 = 54.0;
const BADGE_PAD_X: f32 = 10.0;
const BADGE_HEIGHT: f32 = 22.0;
const ROW_SIDE_PADDING: f32 = 14.0;

/// Lays out a single line of text that ends with an ellipsis instead of overflowing.
fn single_line(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<egui::Galley> {
    let mut job = LayoutJob::simple(text.to_owned(), font, color, max_width.max(8.0));
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    ui.fonts(|fonts| fonts.layout_job(job))
}

pub fn render_result_list(
    ui: &mut Ui,
    theme: &YasalTheme,
    results: &[QueryResult],
    selected_index: usize,
    icon_cache: &mut IconCache,
) -> Option<ResultListAction> {
    if results.is_empty() {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 120.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Tidak ada hasil",
            FontId::proportional(14.0),
            theme.text_secondary,
        );
        return None;
    }

    let row_height = ROW_HEIGHT;
    let mut triggered_action = None;

    // Only scroll when the selection actually changed. Calling scroll_to_rect on every
    // frame keeps the ScrollArea animating and forces a continuous repaint loop.
    let scroll_state_id = Id::new("yasal_result_list_last_scrolled");
    // The key also covers the result set so a new query scrolls back to the top.
    let scroll_key = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        selected_index.hash(&mut hasher);
        for item in results {
            item.id.hash(&mut hasher);
        }
        hasher.finish()
    };
    let last_scrolled: Option<u64> = ui.ctx().data(|data| data.get_temp(scroll_state_id));
    let should_scroll = last_scrolled != Some(scroll_key);

    ui.vertical(|ui| {
        for (idx, item) in results.iter().enumerate() {
            let (row_rect, row_response) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), row_height), Sense::click());

            let is_selected = idx == selected_index;
            let is_hovered = row_response.hovered();

            if is_selected && should_scroll {
                ui.scroll_to_rect(row_rect, Some(Align::Center));
            }

            // Background & selection highlight
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

            // Icon
            let icon_rect = Rect::from_center_size(
                row_rect.left_center() + Vec2::new(ROW_SIDE_PADDING + ICON_SIZE / 2.0, 0.0),
                Vec2::splat(ICON_SIZE),
            );

            let native_texture = match &item.icon {
                ItemIcon::App(path) | ItemIcon::File(path) => {
                    icon_cache.texture_for(ui.ctx(), path)
                }
                ItemIcon::Symbolic(_) => None,
            };

            if let Some(texture_id) = native_texture {
                ui.painter().image(
                    texture_id,
                    icon_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                // Placeholder tile while the native icon is loading / for symbolic items.
                ui.painter().rect(
                    icon_rect,
                    Rounding::same(ICON_CORNER_RADIUS),
                    theme.surface_color,
                    Stroke::new(0.5_f32, theme.border_color),
                );
                match &item.icon {
                    ItemIcon::App(_) | ItemIcon::File(_) => {
                        ui.painter()
                            .circle_filled(icon_rect.center(), 3.0, theme.border_color);
                    }
                    ItemIcon::Symbolic(symbol) => {
                        paint_icon(
                            ui.painter(),
                            icon_rect.center(),
                            symbol,
                            16.0,
                            theme.text_primary,
                        );
                    }
                }
            }

            // Category badge (outlined pill) on the right side
            let badge_galley = ui.painter().layout_no_wrap(
                item.category.clone(),
                FontId::proportional(11.0),
                theme.text_secondary,
            );
            let badge_width = badge_galley.size().x + BADGE_PAD_X * 2.0;
            let badge_rect = Rect::from_center_size(
                row_rect.right_center() + Vec2::new(-ROW_SIDE_PADDING - badge_width / 2.0, 0.0),
                Vec2::new(badge_width, BADGE_HEIGHT),
            );
            ui.painter().rect(
                badge_rect,
                Rounding::same(BADGE_HEIGHT / 2.0),
                Color32::TRANSPARENT,
                Stroke::new(1.0_f32, theme.border_color),
            );
            ui.painter().galley(
                badge_rect.center() - badge_galley.size() / 2.0,
                badge_galley,
                theme.text_secondary,
            );

            // Title and subtitle, truncated with an ellipsis before the badge.
            let text_max_width = (badge_rect.left() - 12.0) - (row_rect.left() + TEXT_LEFT);
            let has_subtitle = item.subtitle.is_some();
            let title_galley = single_line(
                ui,
                &item.title,
                FontId::proportional(14.0),
                theme.text_primary,
                text_max_width,
            );
            let title_y = if has_subtitle {
                row_rect.center().y - 7.5
            } else {
                row_rect.center().y
            };
            ui.painter().galley(
                Pos2::new(
                    row_rect.left() + TEXT_LEFT,
                    title_y - title_galley.size().y / 2.0,
                ),
                title_galley,
                theme.text_primary,
            );

            if let Some(ref subtitle) = item.subtitle {
                let sub_galley = single_line(
                    ui,
                    subtitle,
                    FontId::proportional(11.0),
                    theme.text_secondary,
                    text_max_width,
                );
                ui.painter().galley(
                    Pos2::new(
                        row_rect.left() + TEXT_LEFT,
                        row_rect.center().y + 9.0 - sub_galley.size().y / 2.0,
                    ),
                    sub_galley,
                    theme.text_secondary,
                );
            }

            if row_response.clicked() {
                triggered_action = Some(ResultListAction::Execute(idx));
            } else if is_hovered && !is_selected && row_response.hover_pos().is_some() {
                // Only react to real pointer movement so a stationary cursor does not
                // override keyboard navigation.
                if ui.input(|i| i.pointer.delta() != Vec2::ZERO) {
                    triggered_action = Some(ResultListAction::Select(idx));
                }
            }
        }
    });

    if should_scroll {
        ui.ctx()
            .data_mut(|data| data.insert_temp(scroll_state_id, scroll_key));
    }

    triggered_action
}
