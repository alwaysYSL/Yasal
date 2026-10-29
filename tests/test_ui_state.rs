#[test]
fn test_escape_key_two_step() {
    let mut query = String::from("visual studio");
    let mut is_open = true;

    // Step 1: Query is not empty -> Esc clears query
    if !query.is_empty() {
        query.clear();
    } else {
        is_open = false;
    }
    assert!(query.is_empty());
    assert!(is_open);

    // Step 2: Query is empty -> Esc closes window
    if !query.is_empty() {
        query.clear();
    } else {
        is_open = false;
    }
    assert!(!is_open);
}

#[test]
fn test_settings_command_filter_normalization() {
    use yasal::ui::settings_view::{settings_filter, settings_query_filter};

    assert_eq!(settings_query_filter("settings"), Some(""));
    assert_eq!(settings_query_filter("SETTINGS hotkey"), Some("hotkey"));
    assert_eq!(settings_query_filter("settings theme"), Some("theme"));
    assert_eq!(settings_query_filter("settings-page"), None);
    assert_eq!(settings_filter("settings"), "");
    assert_eq!(settings_filter("settings hotkey"), "hotkey");
    assert_eq!(settings_filter("hotkey"), "hotkey");
}

#[test]
fn test_filtered_settings_count() {
    use yasal::config::AppConfig;
    use yasal::ui::settings_view::filtered_setting_count;

    let config = AppConfig::default();
    assert_eq!(filtered_setting_count(&config, "settings"), 15);
    assert_eq!(filtered_setting_count(&config, "settings hotkey"), 1);
    assert_eq!(filtered_setting_count(&config, "theme"), 1);
    assert_eq!(filtered_setting_count(&config, "does-not-exist"), 0);
}

#[test]
fn test_action_filter_preserves_original_indices() {
    use std::sync::Arc;
    use yasal::core::plugin::ItemAction;
    use yasal::ui::action_panel::filtered_action_indices;

    let actions = vec![
        ItemAction {
            label: "Jalankan sebagai administrator".to_string(),
            icon: "A",
            shortcut_hint: None,
            action: Arc::new(|| {}),
        },
        ItemAction {
            label: "Buka lokasi file".to_string(),
            icon: "F",
            shortcut_hint: None,
            action: Arc::new(|| {}),
        },
        ItemAction {
            label: "Salin path".to_string(),
            icon: "C",
            shortcut_hint: None,
            action: Arc::new(|| {}),
        },
    ];

    assert_eq!(filtered_action_indices(&actions, ""), vec![0, 1, 2]);
    assert_eq!(filtered_action_indices(&actions, "LOKASI"), vec![1]);
    assert_eq!(filtered_action_indices(&actions, "salin"), vec![2]);
}

#[test]
fn test_hotkey_setting_persists_and_about_does_not_toggle() {
    use yasal::config::AppConfig;
    use yasal::ui::settings_view::toggle_setting_at;

    let mut config = AppConfig::default();
    assert_eq!(toggle_setting_at(&mut config, 0, ""), Some("hotkey"));
    assert_eq!(config.hotkey, "Ctrl+Space");
    // Row 13 is the folder picker, row 14 is "Tentang Yasal", which does not toggle.
    assert_eq!(toggle_setting_at(&mut config, 14, ""), None);
}

#[test]
fn test_window_switcher_setting_toggles() {
    use yasal::config::AppConfig;
    use yasal::ui::settings_view::toggle_setting_at;

    let mut config = AppConfig::default();
    assert!(config.enable_window_switcher);
    assert_eq!(toggle_setting_at(&mut config, 11, ""), Some("windows"));
    assert!(!config.enable_window_switcher);
}

#[test]
fn test_fade_in_opacity_is_monotonic_and_bounded() {
    use std::time::Duration;
    use yasal::theme::{fade_in_opacity, FADE_IN_DURATION};

    assert_eq!(fade_in_opacity(Duration::ZERO), 0.0);
    assert_eq!(fade_in_opacity(FADE_IN_DURATION), 1.0);
    assert_eq!(fade_in_opacity(FADE_IN_DURATION * 5), 1.0);

    let mut previous = 0.0;
    for step in 1..=10 {
        let value = fade_in_opacity(FADE_IN_DURATION * step / 10);
        assert!(value >= previous, "opacity must never decrease");
        assert!((0.0..=1.0).contains(&value));
        previous = value;
    }
}

#[test]
fn test_hotkey_presets_cycle_and_recover_from_unknown_values() {
    use yasal::ui::settings_view::{next_hotkey_preset, HOTKEY_PRESETS};

    assert_eq!(next_hotkey_preset("Alt+Space"), "Ctrl+Space");
    assert_eq!(next_hotkey_preset("alt+space"), "Ctrl+Space");
    // The last preset wraps around to the first.
    let last = HOTKEY_PRESETS[HOTKEY_PRESETS.len() - 1];
    assert_eq!(next_hotkey_preset(last), HOTKEY_PRESETS[0]);
    // A value that is not a preset (hand-edited config) restarts the cycle.
    assert_eq!(next_hotkey_preset("Win+Q"), HOTKEY_PRESETS[0]);
}
