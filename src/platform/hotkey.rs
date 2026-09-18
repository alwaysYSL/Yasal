use crate::platform::win32;
use egui::Context;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
};

static TRIGGERED: AtomicBool = AtomicBool::new(false);
static LISTENER_SPAWNED: AtomicBool = AtomicBool::new(false);
static HWND_STORED: AtomicIsize = AtomicIsize::new(0);

// Global context holder to wake up egui event loop from the background thread
static EGUI_CTX: Mutex<Option<Context>> = Mutex::new(None);

const HOTKEY_ID_ALT_SPACE: i32 = 1001;
const HOTKEY_ID_CTRL_SPACE: i32 = 1002;

pub struct HotkeyManager {
    _thread: std::thread::JoinHandle<()>,
}

impl HotkeyManager {
    /// Stores the main window HWND for native foreground activation
    pub fn set_window_hwnd(hwnd: isize) {
        HWND_STORED.store(hwnd, Ordering::SeqCst);
    }

    /// Creates and registers the global hotkey listener via Win32 RegisterHotKey.
    pub fn new(ctx: Context) -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(mut guard) = EGUI_CTX.lock() {
            *guard = Some(ctx);
        }

        if LISTENER_SPAWNED.swap(true, Ordering::SeqCst) {
            // Already spawned
        }

        let handle = std::thread::Builder::new()
            .name("yasal-hotkey-listener".to_string())
            .spawn(move || unsafe {
                // Register Alt+Space with MOD_NOREPEAT to prevent OS System Menu (SC_KEYMENU)
                let alt_space_ok = RegisterHotKey(
                    None,
                    HOTKEY_ID_ALT_SPACE,
                    HOT_KEY_MODIFIERS(MOD_ALT.0 | MOD_NOREPEAT.0),
                    VK_SPACE.0 as u32,
                )
                .is_ok();

                // Also register Ctrl+Space as an alternative companion hotkey
                let _ = RegisterHotKey(
                    None,
                    HOTKEY_ID_CTRL_SPACE,
                    HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_NOREPEAT.0),
                    VK_SPACE.0 as u32,
                );

                if !alt_space_ok {
                    // Fallback registration without MOD_NOREPEAT
                    let _ = RegisterHotKey(
                        None,
                        HOTKEY_ID_ALT_SPACE,
                        MOD_ALT,
                        VK_SPACE.0 as u32,
                    );
                }

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                    if msg.message == WM_HOTKEY {
                        TRIGGERED.store(true, Ordering::SeqCst);

                        // 1. Immediately wake up egui event loop
                        if let Ok(guard) = EGUI_CTX.lock() {
                            if let Some(ref ctx) = *guard {
                                ctx.request_repaint();
                            }
                        }

                        // 2. Bring window to the foreground natively
                        let hwnd = HWND_STORED.load(Ordering::SeqCst);
                        if hwnd != 0 {
                            win32::show_and_focus_window(hwnd);
                        }
                    }

                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                let _ = UnregisterHotKey(None, HOTKEY_ID_ALT_SPACE);
                let _ = UnregisterHotKey(None, HOTKEY_ID_CTRL_SPACE);
            })?;

        Ok(Self { _thread: handle })
    }

    /// Checks and consumes any pending hotkey press event.
    pub fn poll_is_pressed(&self) -> bool {
        TRIGGERED.swap(false, Ordering::SeqCst)
    }
}
