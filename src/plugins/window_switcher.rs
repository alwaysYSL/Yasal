use crate::core::matcher::Matcher;
use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use crate::platform::win32::{self, WindowInfo};
use std::path::Path;
use std::sync::Arc;

const MAX_RESULTS: usize = 20;

/// Switches to an open window. Triggered with the `w` prefix: `w` lists every window
/// (most recently used first) and `w <text>` filters by window title or program name.
pub struct WindowSwitcherPlugin {
    matcher: Matcher,
}

impl WindowSwitcherPlugin {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(),
        }
    }
}

impl Default for WindowSwitcherPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// File name without directory, e.g. `chrome.exe`.
fn program_name(exe_path: &str) -> String {
    Path::new(exe_path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| exe_path.to_string())
}

/// Ranks `windows` against `query`. With an empty query the original order (most
/// recently used first) is kept; otherwise windows are ordered by fuzzy score, with the
/// original order breaking ties. Windows that do not match are dropped.
fn build_results(windows: Vec<WindowInfo>, query: &str, matcher: &Matcher) -> Vec<QueryResult> {
    let mut ranked: Vec<(u32, usize, WindowInfo)> = windows
        .into_iter()
        .enumerate()
        .filter_map(|(order, window)| {
            let score = if query.is_empty() {
                Some(1_000u32.saturating_sub(order as u32))
            } else {
                let title_score = matcher.fuzzy_score(query, &window.title);
                let program_score = window
                    .exe_path
                    .as_deref()
                    .and_then(|path| matcher.fuzzy_score(query, &program_name(path)))
                    // A program-name match is weaker evidence than a title match.
                    .map(|score| score / 2);
                title_score.max(program_score)
            };
            score.map(|score| (score, order, window))
        })
        .collect();

    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    ranked.truncate(MAX_RESULTS);

    ranked
        .into_iter()
        .map(|(score, _, window)| window_result(window, score))
        .collect()
}

fn window_result(window: WindowInfo, score: u32) -> QueryResult {
    let hwnd = window.hwnd;
    let program = window.exe_path.as_deref().map(program_name);

    let mut subtitle = program.clone().unwrap_or_else(|| "Jendela".to_string());
    if window.minimized {
        subtitle.push_str(" · diminimalkan");
    }

    // Ids are shared per program so frecency history stays small and bounded.
    let id = format!(
        "window:{}",
        program.as_deref().unwrap_or("unknown").to_ascii_lowercase()
    );

    let icon = match window.exe_path {
        Some(path) => ItemIcon::App(path),
        None => ItemIcon::Symbolic("window"),
    };

    QueryResult {
        id,
        title: window.title,
        subtitle: Some(subtitle),
        category: "Jendela".to_string(),
        icon,
        score,
        primary_action: Arc::new(move || win32::focus_window(hwnd)),
        secondary_action: None,
        additional_actions: vec![ItemAction {
            label: "Tutup jendela".to_string(),
            icon: "close",
            shortcut_hint: None,
            action: Arc::new(move || win32::close_window(hwnd)),
        }],
    }
}

impl Plugin for WindowSwitcherPlugin {
    fn id(&self) -> &str {
        "window_switcher"
    }

    fn name(&self) -> &str {
        "Window Switcher"
    }

    fn prefix(&self) -> Option<&str> {
        Some("w")
    }

    fn is_global(&self) -> bool {
        false
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        build_results(win32::list_top_level_windows(), input.trim(), &self.matcher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(hwnd: isize, title: &str, exe: Option<&str>, minimized: bool) -> WindowInfo {
        WindowInfo {
            hwnd,
            title: title.to_string(),
            exe_path: exe.map(str::to_string),
            minimized,
        }
    }

    fn sample() -> Vec<WindowInfo> {
        vec![
            window(
                1,
                "notes.txt - Notepad",
                Some(r"C:\Windows\notepad.exe"),
                false,
            ),
            window(2, "Inbox - Mail", Some(r"C:\Apps\Mail\mail.exe"), true),
            window(
                3,
                "Yasal - Visual Studio Code",
                Some(r"C:\Code\Code.exe"),
                false,
            ),
        ]
    }

    #[test]
    fn empty_query_keeps_most_recently_used_order() {
        let results = build_results(sample(), "", &Matcher::new());
        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "notes.txt - Notepad",
                "Inbox - Mail",
                "Yasal - Visual Studio Code"
            ]
        );
        assert!(results.windows(2).all(|pair| pair[0].score > pair[1].score));
    }

    #[test]
    fn query_filters_by_title_and_by_program_name() {
        let by_title = build_results(sample(), "inbox", &Matcher::new());
        assert_eq!(by_title.len(), 1);
        assert_eq!(by_title[0].title, "Inbox - Mail");

        let by_program = build_results(sample(), "notepad", &Matcher::new());
        assert_eq!(by_program[0].title, "notes.txt - Notepad");

        assert!(build_results(sample(), "zzzzqq", &Matcher::new()).is_empty());
    }

    #[test]
    fn results_describe_minimized_state_and_offer_a_close_action() {
        let results = build_results(sample(), "", &Matcher::new());
        assert_eq!(results[0].subtitle.as_deref(), Some("notepad.exe"));
        assert_eq!(
            results[1].subtitle.as_deref(),
            Some("mail.exe · diminimalkan")
        );
        assert_eq!(results[0].category, "Jendela");
        assert_eq!(results[0].additional_actions.len(), 1);
        assert_eq!(results[0].additional_actions[0].label, "Tutup jendela");
        assert_eq!(results[0].id, "window:notepad.exe");
    }

    #[test]
    fn window_without_a_known_program_uses_a_symbolic_icon() {
        let results = build_results(vec![window(9, "Mystery", None, false)], "", &Matcher::new());
        assert_eq!(results[0].icon, ItemIcon::Symbolic("window"));
        assert_eq!(results[0].id, "window:unknown");
    }

    #[test]
    fn results_are_capped() {
        let many: Vec<WindowInfo> = (0..40)
            .map(|index| window(index, &format!("Window {index}"), None, false))
            .collect();
        assert_eq!(build_results(many, "", &Matcher::new()).len(), MAX_RESULTS);
    }
}
