use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

static TRIGGERED: AtomicBool = AtomicBool::new(false);
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

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
            // Check if Alt (VK_MENU) is pressed
            let alt_down = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
            // Check if Ctrl is pressed as alternative
            let ctrl_down = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;

            if alt_down || ctrl_down {
                TRIGGERED.store(true, Ordering::SeqCst);
                // Return 1 to consume Alt+Space so Windows system menu doesn't open
                if alt_down {
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

impl HotkeyManager {
    /// Spawns a dedicated background thread with a Win32 Low-Level Keyboard Hook.
    /// This is the most reliable way on Windows to intercept Alt+Space and Ctrl+Space
    /// without any OS reservation or conflict issues.
    pub fn new_alt_space() -> Result<Self, Box<dyn std::error::Error>> {
        if HOOK_INSTALLED.swap(true, Ordering::SeqCst) {
            // Already installed
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
                        // Keep hook message pump alive
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
