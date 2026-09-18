use yasal::config::{AppConfig, ThemePreference};

#[test]
fn test_config_defaults() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.hotkey, "Alt+Space");
    assert_eq!(cfg.theme, ThemePreference::Auto);
    assert_eq!(cfg.search_engine, "Google");
    assert!(cfg.enable_calculator);
}

#[test]
fn test_config_serde_roundtrip() {
    let mut cfg = AppConfig::default();
    cfg.hotkey = "Ctrl+Space".to_string();
    cfg.theme = ThemePreference::Dark;

    let json = serde_json::to_string(&cfg).unwrap();
    let parsed: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.hotkey, "Ctrl+Space");
    assert_eq!(parsed.theme, ThemePreference::Dark);
}
