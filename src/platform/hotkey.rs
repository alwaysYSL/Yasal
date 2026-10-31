use crate::platform::win32;
use egui::Context;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use std::sync::mpsc;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PeekMessageW, PostThreadMessageW, TranslateMessage, MSG,
    PM_NOREMOVE, WM_APP, WM_HOTKEY,
};

static TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
static LISTENER_SPAWNED: AtomicBool = AtomicBool::new(false);
static WINDOW_HWND: AtomicIsize = AtomicIsize::new(0);
static LISTENER_THREAD_ID: AtomicU32 = AtomicU32::new(0);

/// Posted to the listener thread to tell it to apply a pending rebind request.
const WM_REBIND_HOTKEY: u32 = WM_APP + 1;

const HOTKEY_ID_PRIMARY: i32 = 1001;
const HOTKEY_ID_FALLBACK: i32 = 1002;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HotkeySpec {
    modifiers: HOT_KEY_MODIFIERS,
    key: u32,
}

fn parse_hotkey(value: &str) -> Option<HotkeySpec> {
    let parts = value.split('+').map(str::trim).collect::<Vec<_>>();
    let (key_name, modifier_names) = parts.split_last()?;
    let mut modifiers = HOT_KEY_MODIFIERS(0);
    for name in modifier_names {
        let flag = match name.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => MOD_CONTROL,
            "alt" => MOD_ALT,
            "shift" => MOD_SHIFT,
            "win" | "windows" => MOD_WIN,
            _ => return None,
        };
        if modifiers.0 & flag.0 != 0 {
            return None;
        }
        modifiers.0 |= flag.0;
    }
    if modifiers.0 == 0 {
        return None;
    }
    let uppercase = key_name.to_ascii_uppercase();
    let key = if uppercase == "SPACE" {
        VK_SPACE.0 as u32
    } else if uppercase.len() == 1 && uppercase.as_bytes()[0].is_ascii_alphanumeric() {
        uppercase.as_bytes()[0] as u32
    } else {
        let number = uppercase
            .strip_prefix('F')
            .and_then(|n| n.parse::<u32>().ok())?;
        if !(1..=12).contains(&number) {
            return None;
        }
        0x70 + number - 1
    };
    Some(HotkeySpec { modifiers, key })
}

unsafe fn register_hotkey(id: i32, spec: HotkeySpec) -> bool {
    RegisterHotKey(
        None,
        id,
        HOT_KEY_MODIFIERS(spec.modifiers.0 | MOD_NOREPEAT.0),
        spec.key,
    )
    .or_else(|_| RegisterHotKey(None, id, spec.modifiers, spec.key))
    .is_ok()
}

/// A request to change the active hotkey, executed on the listener thread because
/// RegisterHotKey(None, ..) binds the hotkey to the thread that registers it.
struct RebindRequest {
    spec: HotkeySpec,
    reply: mpsc::SyncSender<Result<(), String>>,
}

/// Replaces the active hotkey; restores the previous one if the new one is unavailable.
unsafe fn rebind(
    active: &mut HotkeySpec,
    active_id: &mut i32,
    new: HotkeySpec,
) -> Result<(), String> {
    if *active == new {
        return Ok(());
    }
    let _ = UnregisterHotKey(None, *active_id);
    if register_hotkey(HOTKEY_ID_PRIMARY, new) {
        *active = new;
        *active_id = HOTKEY_ID_PRIMARY;
        return Ok(());
    }
    // Put the previous hotkey back so the palette stays reachable.
    let _ = register_hotkey(*active_id, *active);
    Err("Kombinasi tombol sedang digunakan aplikasi lain".to_string())
}

pub struct HotkeyManager {
    _thread: std::thread::JoinHandle<()>,
    warning: Option<String>,
    requests: mpsc::Sender<RebindRequest>,
}

impl HotkeyManager {
    pub fn set_window_hwnd(hwnd: isize) {
        WINDOW_HWND.store(hwnd, Ordering::SeqCst);
    }

    /// Creates and registers the global hotkey listener via Win32 RegisterHotKey.
    pub fn new(ctx: Context, preferred_hotkey: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let primary = parse_hotkey(preferred_hotkey).unwrap_or(HotkeySpec {
            modifiers: MOD_ALT,
            key: VK_SPACE.0 as u32,
        });
        let fallback_label = if primary.modifiers == MOD_ALT && primary.key == VK_SPACE.0 as u32 {
            "Ctrl+Space"
        } else {
            "Alt+Space"
        };
        let fallback = parse_hotkey(fallback_label).expect("built-in fallback is valid");
        let preferred_label = preferred_hotkey.to_string();
        if LISTENER_SPAWNED.swap(true, Ordering::SeqCst) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "global hotkey listener is already active",
            )
            .into());
        }

        let (registration_tx, registration_rx) = mpsc::sync_channel(1);
        let (request_tx, request_rx) = mpsc::channel::<RebindRequest>();
        let ctx_clone = ctx.clone();
        let handle = match std::thread::Builder::new()
            .name("yasal-hotkey-listener".to_string())
            .spawn(move || unsafe {
                // Force this thread's message queue to exist so PostThreadMessageW works.
                let mut probe = MSG::default();
                let _ = PeekMessageW(&mut probe, HWND::default(), 0, 0, PM_NOREMOVE);
                LISTENER_THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);

                let primary_ok = register_hotkey(HOTKEY_ID_PRIMARY, primary);
                let fallback_ok = !primary_ok && register_hotkey(HOTKEY_ID_FALLBACK, fallback);

                if !primary_ok && !fallback_ok {
                    let _ = registration_tx.send(Err(
                        format!("{preferred_label} dan {fallback_label} sedang digunakan aplikasi lain"),
                    ));
                    LISTENER_SPAWNED.store(false, Ordering::SeqCst);
                    return;
                }

                let warning = fallback_ok.then(|| {
                    format!("{preferred_label} sedang digunakan aplikasi lain. Yasal memakai {fallback_label}.")
                });
                let _ = registration_tx.send(Ok(warning));

                let mut active = if primary_ok { primary } else { fallback };
                let mut active_id = if primary_ok {
                    HOTKEY_ID_PRIMARY
                } else {
                    HOTKEY_ID_FALLBACK
                };

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                    if msg.message == WM_REBIND_HOTKEY {
                        while let Ok(request) = request_rx.try_recv() {
                            let result = rebind(&mut active, &mut active_id, request.spec);
                            let _ = request.reply.send(result);
                        }
                        continue;
                    }

                    if msg.message == WM_HOTKEY {
                        TRIGGER_COUNT.fetch_add(1, Ordering::SeqCst);

                        // CPU Optimization: Wake up egui event loop
                        ctx_clone.request_repaint();

                        let hwnd = WINDOW_HWND.load(Ordering::SeqCst);
                        let was_visible = win32::is_window_visible(hwnd);
                        if was_visible {
                            win32::request_window_redraw(hwnd);
                        } else {
                            win32::show_and_focus_window(hwnd);
                        }
                    }

                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                let _ = UnregisterHotKey(None, HOTKEY_ID_PRIMARY);
                let _ = UnregisterHotKey(None, HOTKEY_ID_FALLBACK);
                LISTENER_THREAD_ID.store(0, Ordering::SeqCst);
                LISTENER_SPAWNED.store(false, Ordering::SeqCst);
            }) {
            Ok(handle) => handle,
            Err(error) => {
                LISTENER_SPAWNED.store(false, Ordering::SeqCst);
                return Err(error.into());
            }
        };

        let warning = match registration_rx.recv()? {
            Ok(warning) => warning,
            Err(message) => {
                return Err(io::Error::new(io::ErrorKind::AddrInUse, message).into());
            }
        };

        Ok(Self {
            _thread: handle,
            warning,
            requests: request_tx,
        })
    }

    /// Switches the global hotkey at runtime, without restarting Yasal.
    ///
    /// On failure (invalid text, or the combination is taken by another app) the
    /// previous hotkey stays active and an explanatory message is returned.
    pub fn rebind(&self, hotkey: &str) -> Result<(), String> {
        let spec =
            parse_hotkey(hotkey).ok_or_else(|| format!("Format pintasan tidak valid: {hotkey}"))?;
        let thread_id = LISTENER_THREAD_ID.load(Ordering::SeqCst);
        if thread_id == 0 {
            return Err("Pendengar pintasan tidak aktif".to_string());
        }

        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.requests
            .send(RebindRequest {
                spec,
                reply: reply_tx,
            })
            .map_err(|_| "Pendengar pintasan sudah berhenti".to_string())?;
        unsafe {
            PostThreadMessageW(thread_id, WM_REBIND_HOTKEY, WPARAM(0), LPARAM(0))
                .map_err(|error| format!("Gagal menghubungi pendengar pintasan: {error}"))?;
        }
        reply_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .map_err(|_| "Pendengar pintasan tidak merespons".to_string())?
    }

    pub fn warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }

    /// Returns and consumes every pending hotkey press without collapsing
    /// rapid hide/show pairs into a single toggle.
    pub fn poll_press_count(&self) -> u32 {
        TRIGGER_COUNT.swap(0, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_configured_hotkeys_and_rejects_invalid_combinations() {
        assert_eq!(parse_hotkey("Ctrl+Shift+Y").unwrap().key, b'Y' as u32);
        assert_eq!(parse_hotkey("alt+f12").unwrap().key, 0x7B);
        assert_eq!(parse_hotkey("Win+Space").unwrap().modifiers, MOD_WIN);
        assert!(parse_hotkey("Space").is_none());
        assert!(parse_hotkey("Ctrl+Ctrl+A").is_none());
        assert!(parse_hotkey("Ctrl+F13").is_none());
        assert!(parse_hotkey("Ctrl+").is_none());
    }
}
