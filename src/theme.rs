use egui::{Color32, Rounding, Stroke, Style, Visuals};

#[derive(Clone, Debug)]
pub struct YasalTheme {
    pub is_dark: bool,
    pub accent_color: Color32,
    pub bg_color: Color32,
    pub surface_color: Color32,
    pub hover_color: Color32,
    pub selection_color: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub border_color: Color32,
}

impl YasalTheme {
    pub fn new(is_dark: bool, (r, g, b): (u8, u8, u8)) -> Self {
        let accent_color = Color32::from_rgb(r, g, b);

        if is_dark {
            Self {
                is_dark: true,
                accent_color,
                bg_color: Color32::from_rgb(30, 31, 34),
                surface_color: Color32::from_rgb(43, 45, 49),
                hover_color: Color32::from_rgb(53, 56, 62),
                selection_color: Color32::from_rgba_premultiplied(
                    r.saturating_div(3),
                    g.saturating_div(3),
                    b.saturating_div(3),
                    180,
                ),
                text_primary: Color32::from_rgb(242, 243, 245),
                text_secondary: Color32::from_rgb(160, 165, 175),
                border_color: Color32::from_rgb(65, 68, 75),
            }
        } else {
            Self {
                is_dark: false,
                accent_color,
                bg_color: Color32::from_rgb(248, 249, 251),
                surface_color: Color32::from_rgb(238, 240, 244),
                hover_color: Color32::from_rgb(228, 231, 237),
                selection_color: Color32::from_rgba_premultiplied(r, g, b, 45),
                text_primary: Color32::from_rgb(24, 25, 28),
                text_secondary: Color32::from_rgb(110, 115, 125),
                border_color: Color32::from_rgb(215, 218, 226),
            }
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = Style::default();
        let mut visuals = if self.is_dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        visuals.window_rounding = Rounding::same(12.0);
        visuals.window_fill = self.bg_color;
        visuals.window_stroke = Stroke::new(1.0_f32, self.border_color);
        visuals.panel_fill = self.bg_color;

        style.visuals = visuals;
        ctx.set_style(style);
    }
}
