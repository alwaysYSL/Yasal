use yasal::config::{AppConfig, ThemePreference};

#[test]
fn test_config_defaults() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.hotkey, "Alt+Space");
    assert_eq!(cfg.theme, ThemePreference::Auto);
    assert_eq!(cfg.search_engine, "Google");
    assert!(cfg.enable_calculator);
    assert!(cfg.enable_file_search);
    assert!(cfg.enable_windows_settings);
    assert!(cfg.enable_clipboard_history);
    assert!(cfg.enable_network_info);
}

#[test]
fn test_config_serde_roundtrip() {
    let cfg = AppConfig {
        hotkey: "Ctrl+Space".to_string(),
        theme: ThemePreference::Dark,
        ..Default::default()
    };

    let json = serde_json::to_string(&cfg).unwrap();
    let parsed: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.hotkey, "Ctrl+Space");
    assert_eq!(parsed.theme, ThemePreference::Dark);
}

#[test]
fn test_config_migrates_missing_phase_two_fields() {
    let legacy = r#"{
        "hotkey": "Alt+Space",
        "theme": "Auto",
        "run_on_startup": false,
        "search_engine": "Google",
        "enable_calculator": true,
        "enable_web_search": true,
        "enable_shell_command": true,
        "enable_system_commands": true
    }"#;

    let parsed: AppConfig = serde_json::from_str(legacy).unwrap();
    assert!(parsed.enable_file_search);
    assert!(parsed.enable_windows_settings);
    assert!(parsed.enable_clipboard_history);
    assert!(parsed.enable_network_info);
}
