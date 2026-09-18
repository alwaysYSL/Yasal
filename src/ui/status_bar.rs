use crate::theme::YasalTheme;
use egui::{Align2, FontId, Sense, Ui, Vec2};

pub enum ViewMode {
    Search,
    ActionPanel,
    Settings,
}

pub fn render_status_bar(
    ui: &mut Ui,
    theme: &YasalTheme,
    view_mode: &ViewMode,
    selected_category: Option<&str>,
) {
    let height = 26.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

    // Top border for status bar
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        (0.8, theme.border_color),
    );

    let hints = match view_mode {
        ViewMode::ActionPanel => "↑↓ Pilih Aksi   ⏎ Jalankan   Esc Kembali",
        ViewMode::Settings => "↑↓ Pilih   ⏎ Ubah/Toggle   Esc Kembali",
        ViewMode::Search => match selected_category {
            Some("Aplikasi") => "↑↓ Navigasi   ⏎ Buka   Ctrl+⏎ Run as Admin   ⇥ Aksi   Esc Tutup",
            Some("Kalkulator") => "↑↓ Navigasi   ⏎ Salin Hasil   Esc Tutup",
            Some("Web") => "↑↓ Navigasi   ⏎ Buka di Browser   Esc Tutup",
            Some("Perintah Shell") => "↑↓ Navigasi   ⏎ Jalankan di Terminal   Esc Tutup",
            Some("Sistem") => "↑↓ Navigasi   ⏎ Jalankan   Esc Tutup",
            _ => "↑↓ Navigasi   ⏎ Buka   ⇥ Aksi   Esc Tutup",
        },
    };

    let text_pos = rect.left_center() + Vec2::new(12.0, 0.0);
    ui.painter().text(
        text_pos,
        Align2::LEFT_CENTER,
        hints,
        FontId::proportional(11.0),
        theme.text_secondary,
    );

    let brand_pos = rect.right_center() + Vec2::new(-12.0, 0.0);
    ui.painter().text(
        brand_pos,
        Align2::RIGHT_CENTER,
        "🌸 Yasal",
        FontId::proportional(11.0),
        theme.accent_color,
    );
}
