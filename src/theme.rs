use egui::{Color32, Rounding, Stroke, Style, Vec2, Visuals};

/// Transparent space around the card; the soft drop shadow is painted inside it.
pub const SHADOW_MARGIN: f32 = 18.0;
/// Width of the visible card.
pub const CARD_WIDTH: f32 = 650.0;
/// Width of the OS window (card plus shadow margin on both sides).
pub const WINDOW_WIDTH: f32 = CARD_WIDTH + SHADOW_MARGIN * 2.0;
pub const MAX_VISIBLE_ITEMS: usize = 8;
pub const ROW_HEIGHT: f32 = 44.0;
pub const ROW_GAP: f32 = 4.0;
pub const WINDOW_CORNER_RADIUS: f32 = 26.0;
pub const WINDOW_BORDER_WIDTH: f32 = 1.0;
pub const ROW_CORNER_RADIUS: f32 = 14.0;
pub const ICON_CORNER_RADIUS: f32 = 8.0;
pub const SEARCH_BAR_HEIGHT: f32 = 50.0;
pub const SEARCH_BAR_PANEL_HEIGHT: f32 = 70.0;
pub const SEARCH_BAR_RADIUS: f32 = SEARCH_BAR_HEIGHT / 2.0;
pub const CONTENT_HORIZONTAL_MARGIN: f32 = 16.0;
pub const CONTENT_VERTICAL_MARGIN: f32 = 8.0;
pub const STATUS_BAR_PANEL_HEIGHT: f32 = 34.0;
pub const STATUS_BAR_CONTENT_HEIGHT: f32 = 26.0;
pub const WINDOW_FIXED_HEIGHT: f32 =
    SEARCH_BAR_PANEL_HEIGHT + STATUS_BAR_PANEL_HEIGHT + CONTENT_VERTICAL_MARGIN * 2.0;
/// Shortest visible card (search bar, one empty row area and footer).
pub const CARD_MIN_HEIGHT: f32 = 186.0;
/// Tallest visible card: eight result rows.
pub const CARD_MAX_HEIGHT: f32 = WINDOW_FIXED_HEIGHT
    + ROW_HEIGHT * MAX_VISIBLE_ITEMS as f32
    + ROW_GAP * (MAX_VISIBLE_ITEMS as f32 - 1.0);
/// Tallest OS window.
pub const MAX_WINDOW_HEIGHT: f32 = CARD_MAX_HEIGHT + SHADOW_MARGIN * 2.0;

/// How long the palette takes to fade in when it is shown.
pub const FADE_IN_DURATION: std::time::Duration = std::time::Duration::from_millis(100);

/// Opacity (0.0..=1.0) of the palette `elapsed` after it was shown, eased out so the
/// fade starts quickly and settles gently.
pub fn fade_in_opacity(elapsed: std::time::Duration) -> f32 {
    let progress = (elapsed.as_secs_f32() / FADE_IN_DURATION.as_secs_f32()).clamp(0.0, 1.0);
    1.0 - (1.0 - progress).powi(3)
}

#[derive(Clone, Debug)]
pub struct YasalTheme {
    pub is_dark: bool,
    pub accent_color: Color32,
    /// Accent variant for small text; darkened in light mode so it stays readable.
    pub accent_text: Color32,
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
                accent_text: accent_color,
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
                accent_text: Color32::from_rgb(
                    (r as f32 * 0.55) as u8,
                    (g as f32 * 0.55) as u8,
                    (b as f32 * 0.55) as u8,
                ),
                bg_color: Color32::from_rgb(248, 249, 251),
                surface_color: Color32::from_rgb(238, 240, 244),
                hover_color: Color32::from_rgb(228, 231, 237),
                // Unmultiplied: premultiplied RGB larger than alpha is additive and blew the
                // selection out to near-white cyan on the light background.
                selection_color: Color32::from_rgba_unmultiplied(r, g, b, 48),
                text_primary: Color32::from_rgb(24, 25, 28),
                text_secondary: Color32::from_rgb(110, 115, 125),
                border_color: Color32::from_rgb(215, 218, 226),
            }
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = Style::default();
        style.spacing.item_spacing.y = ROW_GAP;
        let mut visuals = if self.is_dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        visuals.window_rounding = Rounding::same(12.0);
        visuals.window_fill = self.bg_color;
        visuals.window_stroke = Stroke::new(1.0_f32, self.border_color);
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;

        style.visuals = visuals;
        ctx.set_style(style);
    }

    pub fn paint_window_frame(&self, ctx: &egui::Context, opacity: f32) {
        // The window is larger than the card: the extra transparent margin holds the shadow.
        let rect = ctx.screen_rect().shrink(SHADOW_MARGIN);
        // Paint into the same layer the panels use (LayerId::background()) and do it
        // BEFORE the panels are shown. A separate, unregistered layer is drawn on top of
        // all registered Background layers, which covered every panel with an opaque card.
        let mut painter = ctx.layer_painter(egui::LayerId::background());
        painter.set_opacity(opacity);

        // The border is a ring made of two fills (outer accent, inner card) instead of a
        // 1px stroke. A stroked path is feathered on both sides, which looked noticeably
        // thicker along the rounded corners than on the straight, pixel-aligned edges.
        // Two fills keep the ring exactly WINDOW_BORDER_WIDTH wide everywhere.
        painter.add(
            egui::epaint::Shadow {
                offset: Vec2::new(0.0, 6.0),
                blur: SHADOW_MARGIN * 1.6,
                spread: 0.0,
                color: Color32::from_black_alpha(if self.is_dark { 150 } else { 70 }),
            }
            .as_shape(rect, Rounding::same(WINDOW_CORNER_RADIUS)),
        );

        painter.rect_filled(
            rect,
            Rounding::same(WINDOW_CORNER_RADIUS),
            self.accent_color,
        );
        painter.rect_filled(
            rect.shrink(WINDOW_BORDER_WIDTH),
            Rounding::same(WINDOW_CORNER_RADIUS - WINDOW_BORDER_WIDTH),
            self.bg_color,
        );
    }
}
