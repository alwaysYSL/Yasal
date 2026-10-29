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
    let mut store = FrecencyStore::new();
    store.record_use("app:vscode");
    store.record_use("app:vscode");
    store.record_use("app:notepad");
    let top = store.get_top_frequent(2);
    assert_eq!(top[0], "app:vscode");
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
