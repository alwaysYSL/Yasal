//! System font setup.
//!
//! The bundled egui fonts are a thin, light-weight face with no arrow or UI icon glyphs,
//! so emoji and arrows rendered as empty boxes and small text looked fragmented.
//! Windows already ships Segoe UI (text) and Segoe Fluent Icons / Segoe MDL2 Assets
//! (icons), so they are loaded from the system instead of being embedded in the exe.

use egui::{Context, FontData, FontDefinitions, FontFamily, Id};
use std::path::PathBuf;

/// Font family holding the Segoe icon glyphs (private-use code points).
pub const ICON_FAMILY: &str = "yasal_icons";

const ICONS_AVAILABLE_KEY: &str = "yasal_icons_available";

fn fonts_dir() -> PathBuf {
    std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\Windows"))
        .join("Fonts")
}

fn read_first(file_names: &[&str]) -> Option<Vec<u8>> {
    let dir = fonts_dir();
    file_names
        .iter()
        .find_map(|name| std::fs::read(dir.join(name)).ok())
}

/// Installs Segoe UI as the primary text font and registers the icon family.
/// Missing fonts are skipped; egui's bundled fonts remain as the fallback.
pub fn install(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    if let Some(bytes) = read_first(&["segoeui.ttf"]) {
        fonts
            .font_data
            .insert("segoe-ui".to_owned(), FontData::from_owned(bytes));
        if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
            family.insert(0, "segoe-ui".to_owned());
        }
    }

    let icons_loaded = match read_first(&["SegoeIcons.ttf", "segmdl2.ttf"]) {
        Some(bytes) => {
            fonts
                .font_data
                .insert("segoe-icons".to_owned(), FontData::from_owned(bytes));
            fonts.families.insert(
                FontFamily::Name(ICON_FAMILY.into()),
                vec!["segoe-icons".to_owned()],
            );
            true
        }
        None => false,
    };

    ctx.set_fonts(fonts);
    ctx.data_mut(|data| data.insert_temp(Id::new(ICONS_AVAILABLE_KEY), icons_loaded));
}

/// True when the icon font was found, so icon glyphs can be drawn safely.
pub fn icons_available(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp::<bool>(Id::new(ICONS_AVAILABLE_KEY)))
        .unwrap_or(false)
}
