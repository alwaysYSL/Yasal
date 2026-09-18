use yasal::core::plugin::Plugin;
use yasal::plugins::app_launcher::AppLauncherPlugin;

#[test]
fn test_app_launcher_query() {
    let launcher = AppLauncherPlugin::new_with_mock(vec![
        (
            "Visual Studio Code".to_string(),
            "C:\\Program Files\\VSCode\\Code.exe".to_string(),
        ),
        ("Notepad".to_string(), "notepad.exe".to_string()),
    ]);

    let results = launcher.query("code");
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Visual Studio Code");
    assert_eq!(results[0].category, "Aplikasi");
    assert!(results[0].secondary_action.is_some());
    assert!(!results[0].additional_actions.is_empty());
}
