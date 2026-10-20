use crate::core::plugin::{ItemAction, ItemIcon, Plugin, QueryResult};
use crate::plugins::app_launcher::set_clipboard_text;
use chrono::{DateTime, Local};
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;
use windows::core::w;
use windows::Win32::Foundation::{HGLOBAL, HWND, LPARAM, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, GetClipboardData, IsClipboardFormatAvailable,
    OpenClipboard, RemoveClipboardFormatListener,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, GetMessageW, PostMessageW, TranslateMessage,
    HMENU, HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_CLIPBOARDUPDATE,
};

const CF_UNICODETEXT: u32 = 13;
const MAX_HISTORY: usize = 50;
const MAX_RESULTS: usize = 8;
const PREVIEW_CHARS: usize = 80;

type SharedHistory = Arc<RwLock<VecDeque<ClipboardEntry>>>;
const WM_STOP_LISTENER: u32 = WM_APP + 1;

#[derive(Clone, Debug)]
struct ClipboardEntry {
    text: String,
    captured_at: DateTime<Local>,
}

pub struct ClipboardHistoryPlugin {
    history: SharedHistory,
    listener_hwnd: isize,
    listener: Option<thread::JoinHandle<()>>,
}

impl ClipboardHistoryPlugin {
    /// Starts the event-driven clipboard listener and returns once Windows has
    /// accepted the listener registration.
    pub fn initialize() -> Result<Self, String> {
        let history = Arc::new(RwLock::new(VecDeque::with_capacity(MAX_HISTORY)));
        let listener_history = Arc::clone(&history);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);

        let listener = thread::Builder::new()
            .name("clipboard-history".to_string())
            .spawn(move || run_listener(listener_history, ready_tx))
            .map_err(|error| format!("failed to start clipboard listener thread: {error}"))?;

        let listener_hwnd = ready_rx
            .recv()
            .map_err(|_| "clipboard listener stopped during initialization".to_string())??;

        Ok(Self {
            history,
            listener_hwnd,
            listener: Some(listener),
        })
    }

    /// Constructor alias intended for plugin registration.
    pub fn new() -> Result<Self, String> {
        Self::initialize()
    }
}

impl Drop for ClipboardHistoryPlugin {
    fn drop(&mut self) {
        // The listener owns the window. Stop it on that thread so its clipboard
        // registration is removed before the plugin can be enabled again.
        let posted = unsafe {
            PostMessageW(
                HWND(self.listener_hwnd as *mut _),
                WM_STOP_LISTENER,
                WPARAM(0),
                LPARAM(0),
            )
        };
        if posted.is_ok() {
            if let Some(listener) = self.listener.take() {
                let _ = listener.join();
            }
        }
    }
}

impl Plugin for ClipboardHistoryPlugin {
    fn id(&self) -> &str {
        "clipboard_history"
    }

    fn name(&self) -> &str {
        "Clipboard History"
    }

    fn prefix(&self) -> Option<&str> {
        Some("cb")
    }

    fn query(&self, input: &str) -> Vec<QueryResult> {
        let needle = input.trim().to_lowercase();
        let entries = match self.history.read() {
            Ok(history) => history
                .iter()
                .filter(|entry| needle.is_empty() || entry.text.to_lowercase().contains(&needle))
                .take(MAX_RESULTS)
                .cloned()
                .collect::<Vec<_>>(),
            Err(_) => return Vec::new(),
        };

        entries
            .into_iter()
            .enumerate()
            .map(|(index, entry)| {
                let text_to_copy = entry.text.clone();
                let copy_action: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                    let _ = set_clipboard_text(&text_to_copy);
                });

                QueryResult {
                    id: format!("clipboard:{}:{index}", entry.captured_at.timestamp_millis()),
                    title: preview(&entry.text, PREVIEW_CHARS),
                    subtitle: Some(format!(
                        "Disalin pukul {} · Enter menyalin kembali",
                        entry.captured_at.format("%H:%M:%S")
                    )),
                    category: "Clipboard".to_string(),
                    icon: ItemIcon::Symbolic("clipboard"),
                    score: 1_000_u32.saturating_sub(index as u32),
                    primary_action: Arc::clone(&copy_action),
                    secondary_action: Some(ItemAction {
                        label: "Salin ke clipboard".to_string(),
                        icon: "📋",
                        shortcut_hint: Some("Enter / Ctrl+Enter".to_string()),
                        action: copy_action,
                    }),
                    additional_actions: Vec::new(),
                }
            })
            .collect()
    }
}

fn run_listener(history: SharedHistory, ready: mpsc::SyncSender<Result<isize, String>>) {
    unsafe {
        let window = match CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("Yasal Clipboard History Listener"),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            HMENU::default(),
            None,
            None,
        ) {
            Ok(window) => window,
            Err(error) => {
                let _ = ready.send(Err(format!("failed to create clipboard listener: {error}")));
                return;
            }
        };

        if let Err(error) = AddClipboardFormatListener(window) {
            let _ = DestroyWindow(window);
            let _ = ready.send(Err(format!(
                "failed to register clipboard listener: {error}"
            )));
            return;
        }

        if ready.send(Ok(window.0 as isize)).is_err() {
            let _ = RemoveClipboardFormatListener(window);
            let _ = DestroyWindow(window);
            return;
        }

        capture_current_text(&history);

        let mut message = MSG::default();
        loop {
            let status = GetMessageW(&mut message, HWND::default(), 0, 0);
            if status.0 <= 0 {
                break;
            }

            if message.message == WM_STOP_LISTENER {
                break;
            }

            if message.message == WM_CLIPBOARDUPDATE {
                capture_current_text(&history);
            }

            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        let _ = RemoveClipboardFormatListener(window);
        let _ = DestroyWindow(window);
    }
}

fn capture_current_text(history: &SharedHistory) {
    if let Some(text) = read_clipboard_text() {
        if let Ok(mut history) = history.write() {
            push_entry(&mut history, text, Local::now());
        }
    }
}

fn read_clipboard_text() -> Option<String> {
    for attempt in 0..3 {
        if let Some(text) = try_read_clipboard_text() {
            return Some(text);
        }
        if attempt < 2 {
            thread::sleep(Duration::from_millis(5));
        }
    }
    None
}

fn try_read_clipboard_text() -> Option<String> {
    unsafe {
        IsClipboardFormatAvailable(CF_UNICODETEXT).ok()?;
        OpenClipboard(None).ok()?;

        let text = (|| {
            let handle = GetClipboardData(CF_UNICODETEXT).ok()?;
            let memory = HGLOBAL(handle.0);
            let byte_len = GlobalSize(memory);
            if byte_len < std::mem::size_of::<u16>() {
                return None;
            }

            let pointer = GlobalLock(memory) as *const u16;
            if pointer.is_null() {
                return None;
            }

            let units = std::slice::from_raw_parts(pointer, byte_len / std::mem::size_of::<u16>());
            let text_len = units
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(units.len());
            let text = String::from_utf16_lossy(&units[..text_len]);
            let _ = GlobalUnlock(memory);

            (!text.is_empty()).then_some(text)
        })();

        let _ = CloseClipboard();
        text
    }
}

fn push_entry(
    history: &mut VecDeque<ClipboardEntry>,
    text: String,
    captured_at: DateTime<Local>,
) -> bool {
    if history.front().is_some_and(|entry| entry.text == text) {
        return false;
    }

    history.push_front(ClipboardEntry { text, captured_at });
    history.truncate(MAX_HISTORY);
    true
}

fn preview(text: &str, max_chars: usize) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = normalized.chars();
    let preview = chars.by_ref().take(max_chars).collect::<String>();

    if chars.next().is_some() {
        format!("{preview}…")
    } else {
        preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;

    #[test]
    fn disabling_listener_releases_window_and_allows_reenable() {
        for _ in 0..2 {
            let plugin = ClipboardHistoryPlugin::initialize().unwrap();
            let hwnd = HWND(plugin.listener_hwnd as *mut _);
            assert!(unsafe { IsWindow(hwnd) }.as_bool());
            drop(plugin);
            assert!(!unsafe { IsWindow(hwnd) }.as_bool());
        }
    }

    #[test]
    fn deduplicates_only_consecutive_text() {
        let mut history = VecDeque::new();

        assert!(push_entry(&mut history, "alpha".to_string(), Local::now()));
        assert!(!push_entry(&mut history, "alpha".to_string(), Local::now()));
        assert!(push_entry(&mut history, "beta".to_string(), Local::now()));
        assert!(push_entry(&mut history, "alpha".to_string(), Local::now()));

        let texts = history
            .iter()
            .map(|entry| entry.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(texts, vec!["alpha", "beta", "alpha"]);
    }

    #[test]
    fn caps_history_and_keeps_newest_entries() {
        let mut history = VecDeque::new();

        for index in 0..(MAX_HISTORY + 7) {
            assert!(push_entry(
                &mut history,
                format!("entry {index}"),
                Local::now()
            ));
        }

        assert_eq!(history.len(), MAX_HISTORY);
        assert_eq!(
            history.front().map(|entry| entry.text.as_str()),
            Some("entry 56")
        );
        assert_eq!(
            history.back().map(|entry| entry.text.as_str()),
            Some("entry 7")
        );
    }

    #[test]
    fn preview_is_unicode_safe_and_collapses_whitespace() {
        assert_eq!(preview("  hello\n\tworld  ", 80), "hello world");
        assert_eq!(preview("😀界éabc", 3), "😀界é…");
        assert_eq!(preview("😀界é", 3), "😀界é");
    }
}
