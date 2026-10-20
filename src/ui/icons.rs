//! Icon glyph lookup and painting.
//!
//! Items name their icon either with a symbolic id (`"calc"`, `"settings"`, ...) or, for
//! older call sites, with an emoji. Both map to Segoe Fluent Icons / MDL2 code points so
//! every icon is a crisp monochrome glyph that follows the theme color.

use crate::fonts::{icons_available, ICON_FAMILY};
use egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2};

/// Maps a symbolic id or emoji to an icon-font code point.
pub fn glyph_for(name: &str) -> Option<char> {
    // Emoji often carry an invisible variation selector (U+FE0F).
    let cleaned: String = name.chars().filter(|&c| c != '\u{FE0F}').collect();
    let code = match cleaned.as_str() {
        "search" | "🔍" => 0xE721,
        "calc" | "🔢" => 0xE8EF,
        "shell" | "⚡" => 0xE756,
        "web" | "🌐" => 0xE774,
        "settings" | "⚙" => 0xE713,
        "file" => 0xE8A5,
        "folder" | "📁" => 0xE8B7,
        "clipboard" => 0xE77F,
        "copy" | "📋" => 0xE8C8,
        "network" => 0xE839,
        "window" => 0xE7C4,
        "close" => 0xE711,
        "power" => 0xE7E8,
        "lock" => 0xE72E,
        "moon" => 0xE708,
        "trash" => 0xE74D,
        "action_panel" | "↩" => 0xE712,
        "hotkey" | "⌨" => 0xE765,
        "startup" | "🚀" => 0xE7B5,
        "theme" | "🎨" => 0xE790,
        "sys" | "shield" | "🛡" => 0xEA18,
        "about" | "info" | "🌸" => 0xE946,
        _ => return None,
    };
    char::from_u32(code)
}

/// Text drawn when the icon font is missing or the name is unknown.
fn fallback_text(name: &str) -> String {
    let cleaned: String = name.chars().filter(|&c| c != '\u{FE0F}').collect();
    if cleaned
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        cleaned
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_default()
    } else {
        cleaned
    }
}

/// Paints an icon centered on `center`.
pub fn paint_icon(painter: &Painter, center: Pos2, name: &str, size: f32, color: Color32) {
    match glyph_for(name) {
        Some(glyph) if icons_available(painter.ctx()) => {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                glyph.to_string(),
                FontId::new(size, FontFamily::Name(ICON_FAMILY.into())),
                color,
            );
        }
        _ => {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                fallback_text(name),
                FontId::proportional(size * 0.8),
                color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbolic_ids_and_legacy_emoji_resolve_to_the_same_glyph() {
        assert_eq!(glyph_for("calc"), glyph_for("🔢"));
        assert_eq!(glyph_for("web"), glyph_for("🌐"));
        assert_eq!(glyph_for("copy"), glyph_for("📋"));
    }

    #[test]
    fn variation_selector_is_ignored() {
        assert_eq!(glyph_for("⚙\u{FE0F}"), glyph_for("settings"));
        assert!(glyph_for("🛡\u{FE0F}").is_some());
    }

    #[test]
    fn unknown_names_have_no_glyph_and_use_a_letter_fallback() {
        assert_eq!(glyph_for("does-not-exist"), None);
        assert_eq!(fallback_text("file"), "F");
    }
}
