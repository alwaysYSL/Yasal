use crate::core::plugin::ItemAction;
use crate::theme::{YasalTheme, ROW_CORNER_RADIUS, ROW_HEIGHT};
use crate::ui::icons::paint_icon;
use egui::{Align, Align2, FontId, Id, Rounding, Sense, Stroke, Ui, Vec2};

pub enum ActionPanelEvent {
    Select(usize),
    Execute(usize),
    Close,
}

pub fn filtered_action_indices(actions: &[ItemAction], filter: &str) -> Vec<usize> {
    let filter = filter.trim().to_lowercase();

    actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| {
            (filter.is_empty() || action.label.to_lowercase().contains(&filter)).then_some(index)
        })
        .collect()
}

pub fn render_action_panel(
    ui: &mut Ui,
    theme: &YasalTheme,
    actions: &[ItemAction],
    selected_index: usize,
) -> Option<ActionPanelEvent> {
    let filter = ui.ctx().data(|data| {
        data.get_temp::<String>(Id::new("yasal_action_filter"))
            .unwrap_or_default()
    });
    render_filtered_action_panel(ui, theme, actions, &filter, selected_index)
}

pub fn render_filtered_action_panel(
    ui: &mut Ui,
    theme: &YasalTheme,
    actions: &[ItemAction],
    filter: &str,
    selected_index: usize,
) -> Option<ActionPanelEvent> {
    if actions.is_empty() {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Tidak ada aksi tambahan untuk item ini",
            FontId::proportional(13.0),
            theme.text_secondary,
        );
        return None;
    }

    let filtered_indices = filtered_action_indices(actions, filter);
    if filtered_indices.is_empty() {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.0), Sense::hover());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Tidak ada aksi yang cocok",
            FontId::proportional(13.0),
            theme.text_secondary,
        );
        return None;
    }

    let row_height = ROW_HEIGHT;
    let mut triggered = None;

    // Scroll only when the selection or filter changed; scrolling every frame keeps
    // the ScrollArea animating and forces continuous repaints.
    let scroll_state_id = Id::new("yasal_action_panel_last_scrolled");
    let scroll_key = (selected_index, filter.to_owned());
    let should_scroll = ui
        .ctx()
        .data(|data| data.get_temp::<(usize, String)>(scroll_state_id))
        .as_ref()
        != Some(&scroll_key);

    ui.vertical(|ui| {
        for &original_index in &filtered_indices {
            let action = &actions[original_index];
            let (row_rect, row_response) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), row_height), Sense::click());

            let is_selected = original_index == selected_index;
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

            // Action icon
            let icon_pos = row_rect.left_center() + Vec2::new(28.0, 0.0);
            paint_icon(
                ui.painter(),
                icon_pos,
                action.icon,
                16.0,
                theme.text_primary,
            );

            // Action label
            let label_pos = row_rect.left_center() + Vec2::new(52.0, 0.0);
            ui.painter().text(
                label_pos,
                Align2::LEFT_CENTER,
                &action.label,
                FontId::proportional(14.0),
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
                    theme.accent_text,
                );
            }

            if row_response.clicked() {
                triggered = Some(ActionPanelEvent::Execute(original_index));
            } else if is_hovered && !is_selected && ui.input(|i| i.pointer.delta() != Vec2::ZERO) {
                // Only follow real pointer movement so a resting cursor does not
                // override keyboard navigation.
                triggered = Some(ActionPanelEvent::Select(original_index));
            }
        }
    });

    if should_scroll {
        ui.ctx()
            .data_mut(|data| data.insert_temp(scroll_state_id, scroll_key));
    }

    triggered
}
