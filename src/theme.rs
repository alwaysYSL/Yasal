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
                bg_color: Color32::from_rgba_premultiplied(28, 28, 30, 220),
                surface_color: Color32::from_rgba_premultiplied(40, 40, 43, 200),
                hover_color: Color32::from_rgba_premultiplied(55, 55, 60, 180),
                selection_color: Color32::from_rgba_premultiplied(r.saturating_div(4), g.saturating_div(4), b.saturating_div(4), 160),
                text_primary: Color32::from_rgb(240, 240, 245),
                text_secondary: Color32::from_rgb(160, 160, 170),
                border_color: Color32::from_rgba_premultiplied(70, 70, 75, 120),
            }
        } else {
            Self {
                is_dark: false,
                accent_color,
                bg_color: Color32::from_rgba_premultiplied(245, 245, 247, 230),
                surface_color: Color32::from_rgba_premultiplied(235, 235, 240, 220),
                hover_color: Color32::from_rgba_premultiplied(225, 225, 232, 200),
                selection_color: Color32::from_rgba_premultiplied(r, g, b, 50),
                text_primary: Color32::from_rgb(25, 25, 30),
                text_secondary: Color32::from_rgb(105, 105, 115),
                border_color: Color32::from_rgba_premultiplied(200, 200, 210, 150),
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
