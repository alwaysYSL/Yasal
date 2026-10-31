use crate::theme::{YasalTheme, STATUS_BAR_CONTENT_HEIGHT};
use egui::{Align2, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};

pub enum ViewMode {
    Search,
    ActionPanel,
    Settings,
}

/// (key cap, label) pairs shown in the footer.
pub fn hints_for(
    view_mode: &ViewMode,
    selected_category: Option<&str>,
) -> Vec<(&'static str, &'static str)> {
    match view_mode {
        ViewMode::ActionPanel => vec![("↑/↓", "Pilih"), ("Enter", "Jalankan"), ("Esc", "Kembali")],
        ViewMode::Settings => vec![("↑/↓", "Pilih"), ("Enter", "Ubah"), ("Esc", "Kembali")],
        ViewMode::Search => match selected_category {
            Some("Kalkulator") | Some("Clipboard") | Some("Jaringan") => {
                vec![("↑/↓", "Pilih"), ("Enter", "Salin"), ("Esc", "Tutup")]
            }
            Some("Web") | Some("Pengaturan Windows") => {
                vec![("↑/↓", "Pilih"), ("Enter", "Buka"), ("Esc", "Tutup")]
            }
            Some("File") | Some("Folder") => vec![
                ("↑/↓", "Pilih"),
                ("Enter", "Buka"),
                ("Ctrl+Enter", "Explorer"),
                ("Esc", "Tutup"),
            ],
            Some("Jendela") => vec![
                ("↑/↓", "Pilih"),
                ("Enter", "Beralih"),
                ("Tab", "Aksi"),
                ("Esc", "Tutup"),
            ],
            Some("Perintah Shell") | Some("Sistem") => {
                vec![("↑/↓", "Pilih"), ("Enter", "Jalankan"), ("Esc", "Tutup")]
            }
            _ => vec![
                ("↑/↓", "Pilih"),
                ("Enter", "Buka"),
                ("Tab", "Aksi"),
                ("Esc", "Tutup"),
            ],
        },
    }
}

pub fn render_status_bar(
    ui: &mut Ui,
    theme: &YasalTheme,
    view_mode: &ViewMode,
    selected_category: Option<&str>,
) {
    let height = STATUS_BAR_CONTENT_HEIGHT;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

    // Top border for status bar
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        (0.8, theme.border_color),
    );

    let cap_font = FontId::proportional(11.0);
    let label_font = FontId::proportional(12.0);
    let cap_height = 20.0;
    let cap_pad_x = 7.0;
    let mut x = rect.left() + 4.0;
    // Slightly below the centre so the key caps keep clear of the divider line.
    let center_y = rect.center().y + 2.0;

    for (key, label) in hints_for(view_mode, selected_category) {
        // The bundled fonts have no arrow glyphs, so the arrow cap is drawn with shapes.
        let is_arrow_cap = key == "↑/↓";
        let key_galley = ui.painter().layout_no_wrap(
            if is_arrow_cap {
                String::new()
            } else {
                key.to_owned()
            },
            cap_font.clone(),
            theme.text_primary,
        );
        let cap_width = if is_arrow_cap {
            38.0
        } else {
            (key_galley.size().x + cap_pad_x * 2.0).max(cap_height)
        };
        let cap_rect = Rect::from_min_size(
            Pos2::new(x, center_y - cap_height / 2.0),
            Vec2::new(cap_width, cap_height),
        );
        ui.painter().rect(
            cap_rect,
            Rounding::same(6.0),
            theme.surface_color,
            Stroke::new(1.0_f32, theme.border_color),
        );
        if is_arrow_cap {
            let stroke = Stroke::new(1.3_f32, theme.text_primary);
            let painter = ui.painter();
            let c = cap_rect.center();
            // Up arrow
            let up = c + Vec2::new(-8.0, 0.0);
            painter.line_segment(
                [up + Vec2::new(0.0, 5.0), up + Vec2::new(0.0, -5.0)],
                stroke,
            );
            painter.line_segment(
                [up + Vec2::new(-3.0, -2.0), up + Vec2::new(0.0, -5.0)],
                stroke,
            );
            painter.line_segment(
                [up + Vec2::new(3.0, -2.0), up + Vec2::new(0.0, -5.0)],
                stroke,
            );
            // Slash
            painter.line_segment([c + Vec2::new(-2.0, 5.0), c + Vec2::new(2.0, -5.0)], stroke);
            // Down arrow
            let down = c + Vec2::new(8.0, 0.0);
            painter.line_segment(
                [down + Vec2::new(0.0, -5.0), down + Vec2::new(0.0, 5.0)],
                stroke,
            );
            painter.line_segment(
                [down + Vec2::new(-3.0, 2.0), down + Vec2::new(0.0, 5.0)],
                stroke,
            );
            painter.line_segment(
                [down + Vec2::new(3.0, 2.0), down + Vec2::new(0.0, 5.0)],
                stroke,
            );
        } else {
            ui.painter().galley(
                cap_rect.center() - key_galley.size() / 2.0,
                key_galley,
                theme.text_primary,
            );
        }

        let label_galley =
            ui.painter()
                .layout_no_wrap(label.to_owned(), label_font.clone(), theme.text_secondary);
        let label_pos = Pos2::new(cap_rect.right() + 6.0, center_y);
        ui.painter().text(
            label_pos,
            Align2::LEFT_CENTER,
            label,
            label_font.clone(),
            theme.text_secondary,
        );
        x = label_pos.x + label_galley.size().x + 14.0;
    }
}
