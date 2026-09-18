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
    assert_eq!(dispatcher.detect_mode_icon("visual studio"), "search");
}
