use yasal::core::dispatcher::Dispatcher;
use yasal::core::frecency::FrecencyStore;
use yasal::core::matcher::Matcher;

#[test]
fn test_fuzzy_matcher() {
    let matcher = Matcher::new();
    let score = matcher.fuzzy_score("vsc", "Visual Studio Code");
    assert!(score.is_some());
    assert!(score.unwrap() > 0);

    let no_match = matcher.fuzzy_score("xyz", "Visual Studio Code");
    assert!(no_match.is_none());
}

#[test]
fn test_frecency_ranking() {
    let mut store = FrecencyStore::in_memory();
    store.record_use("app:vscode");
    store.record_use("app:vscode");
    store.record_use("app:notepad");
    let top = store.get_top_frequent(2);
    assert_eq!(top[0], "app:vscode");
}

#[test]
fn empty_state_ranks_by_frecency_and_skips_unknown_ids() {
    use std::sync::Arc;
    use yasal::plugins::app_launcher::AppLauncherPlugin;

    // More apps than the launcher returns for an empty query, so the ones the user
    // opens most are not among its first results.
    let apps: Vec<(String, String)> = (0..12)
        .map(|i| (format!("App {i:02}"), format!(r"C:\apps\app{i:02}.lnk")))
        .collect();
    let mut store = FrecencyStore::in_memory();
    for _ in 0..3 {
        store.record_use("app:gone"); // no plugin returns this id
    }
    store.record_use("app:app 11");
    store.record_use("app:app 11");
    store.record_use("app:app 10");

    let mut dispatcher = Dispatcher::with_frecency(store);
    dispatcher.register(Arc::new(AppLauncherPlugin::new_with_mock(apps)));

    let ids: Vec<String> = dispatcher.dispatch("").into_iter().map(|r| r.id).collect();
    assert_eq!(ids, ["app:app 11", "app:app 10"]);
}

#[test]
fn test_dispatcher_mode_detection() {
    let dispatcher = Dispatcher::new();
    assert_eq!(dispatcher.detect_mode_icon("= 25 * 4"), "calc");
    assert_eq!(dispatcher.detect_mode_icon("> ipconfig"), "shell");
    assert_eq!(dispatcher.detect_mode_icon("?? rustlang"), "web");
    assert_eq!(dispatcher.detect_mode_icon("settings"), "settings");
    assert_eq!(dispatcher.detect_mode_icon("$ bluetooth"), "settings");
    assert_eq!(dispatcher.detect_mode_icon("file laporan"), "file");
    assert_eq!(dispatcher.detect_mode_icon("cb invoice"), "clipboard");
    assert_eq!(dispatcher.detect_mode_icon("network"), "network");
    assert_eq!(dispatcher.detect_mode_icon("visual studio"), "search");
}

#[test]
fn test_dispatcher_routes_calculator_and_unit_conversions() {
    use std::sync::Arc;
    use yasal::core::dispatcher::Dispatcher;
    use yasal::plugins::calculator::CalculatorPlugin;

    let mut dispatcher = Dispatcher::new();
    dispatcher.register(Arc::new(CalculatorPlugin::new()));

    // Prefixed: no operator needed once the user typed "=".
    assert_eq!(dispatcher.dispatch("= math::sqrt(16)")[0].title, "= 4");
    // Real division through the full dispatch path.
    assert_eq!(dispatcher.dispatch("10/4")[0].title, "= 2.5");
    // Conversions are found by unprefixed global search too.
    assert_eq!(dispatcher.dispatch("5 km to mi")[0].title, "= 3.106856 mi");
    // Ordinary words and incompatible units must not produce calculator noise.
    assert!(dispatcher.dispatch("word to pdf").is_empty());
    assert!(dispatcher.dispatch("5 km to kg").is_empty());
}
