use egui::Context;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, KBDLLHOOKSTRUCT,
    MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

static TRIGGERED: AtomicBool = AtomicBool::new(false);
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

// Global context holder to wake up egui event loop from the background hook
static EGUI_CTX: Mutex<Option<Context>> = Mutex::new(None);

pub struct HotkeyManager {
    _thread: std::thread::JoinHandle<()>,
}

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 && (wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN) {
        let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        if kbd.vkCode == VK_SPACE.0 as u32 {
            let alt_down = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
            let ctrl_down = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;

            if alt_down || ctrl_down {
                TRIGGERED.store(true, Ordering::SeqCst);

                // Wake up egui event loop immediately even when window is hidden/sleeping
                if let Ok(guard) = EGUI_CTX.lock() {
                    if let Some(ref ctx) = *guard {
                        ctx.request_repaint();
                    }
                }

                // Consume Alt+Space so Windows system menu doesn't pop up
                if alt_down {
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

impl HotkeyManager {
    /// Creates and registers the global hotkey listener, storing egui::Context to wake up the UI.
    pub fn new(ctx: Context) -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(mut guard) = EGUI_CTX.lock() {
            *guard = Some(ctx);
        }

        if HOOK_INSTALLED.swap(true, Ordering::SeqCst) {
            // Already spawned
        }

        let handle = std::thread::Builder::new()
            .name("yasal-hotkey-hook".to_string())
            .spawn(move || unsafe {
                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(low_level_keyboard_proc),
                    HMODULE::default(),
                    0,
                );

                if let Ok(hook) = hook {
                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                        // Message pump for low-level hook
                    }
                    let _ = UnhookWindowsHookEx(hook);
                }
            })?;

        Ok(Self { _thread: handle })
    }

    /// Checks and consumes any pending hotkey press event.
    pub fn poll_is_pressed(&self) -> bool {
        TRIGGERED.swap(false, Ordering::SeqCst)
    }
}
