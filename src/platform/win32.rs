use windows::core::HSTRING;
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;
use windows::Win32::System::Power::SetSuspendState;
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ, REG_DWORD,
};
use windows::Win32::System::Shutdown::{
    ExitWindowsEx, LockWorkStation, EWX_FORCEIFHUNG, EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN,
    SHUTDOWN_REASON,
};
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::Shell::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, MessageBoxW, SetForegroundWindow, ShowWindow, IDOK, MB_ICONWARNING,
    MB_OKCANCEL, SW_HIDE, SW_SHOW,
};

/// Forces Windows to trim the process working set, dropping unused RAM pages.
/// Typical effect: active memory drops from ~20-30 MB down to < 5 MB on window dismiss.
pub fn trim_memory() {
    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
}

/// Activates and brings the specified Win32 window to the foreground.
pub fn show_and_focus_window(hwnd_raw: isize) {
    if hwnd_raw == 0 {
        return;
    }
    unsafe {
        let hwnd = HWND(hwnd_raw as *mut _);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = BringWindowToTop(hwnd);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(hwnd);
    }
}

/// Hides the specified Win32 window and trims working set memory immediately.
pub fn hide_window(hwnd_raw: isize) {
    if hwnd_raw != 0 {
        unsafe {
            let hwnd = HWND(hwnd_raw as *mut _);
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }
    trim_memory();
}

/// Retrieves the active Windows accent color in RGB format.
/// Falls back to default Windows 11 Blue (0, 120, 215) on failure.
pub fn get_system_accent_color() -> (u8, u8, u8) {
    let mut colorization: u32 = 0;
    let mut opaque: BOOL = Default::default();
    unsafe {
        if DwmGetColorizationColor(&mut colorization, &mut opaque).is_ok() {
            let r = ((colorization >> 16) & 0xFF) as u8;
            let g = ((colorization >> 8) & 0xFF) as u8;
            let b = (colorization & 0xFF) as u8;
            return (r, g, b);
        }
    }
    (0, 120, 215)
}

/// Checks if Windows is currently set to Dark Mode for applications.
pub fn is_system_dark_mode() -> bool {
    let subkey = HSTRING::from("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let value_name = HSTRING::from("AppsUseLightTheme");
    let mut hkey = Default::default();

    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, &subkey, 0, KEY_READ, &mut hkey).is_ok() {
            let mut data: u32 = 0;
            let mut data_len = std::mem::size_of::<u32>() as u32;
            let mut reg_type = REG_DWORD;
            let res = RegQueryValueExW(
                hkey,
                &value_name,
                None,
                Some(&mut reg_type),
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut data_len),
            );
            let _ = RegCloseKey(hkey);
            if res.is_ok() {
                return data == 0;
            }
        }
    }
    true // Default fallback to dark mode
}

/// Shows a native Windows confirmation message box for destructive actions.
pub fn show_confirm_dialog(title: &str, message: &str) -> bool {
    let title_w = HSTRING::from(title);
    let msg_w = HSTRING::from(message);
    unsafe {
        let result = MessageBoxW(None, &msg_w, &title_w, MB_OKCANCEL | MB_ICONWARNING);
        result == IDOK
    }
}

/// Empties the Windows Recycle Bin silently.
pub fn empty_recycle_bin() -> bool {
    unsafe {
        SHEmptyRecycleBinW(None, None, SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI).is_ok()
    }
}

/// Locks the current Windows workstation.
pub fn lock_workstation() -> bool {
    unsafe { LockWorkStation().is_ok() }
}

/// Puts the system into sleep state.
pub fn sleep_system() -> bool {
    unsafe { SetSuspendState(false, true, false).as_bool() }
}

/// Shuts down the computer.
pub fn shutdown_system() -> bool {
    unsafe {
        ExitWindowsEx(EWX_SHUTDOWN | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0)).is_ok()
    }
}

/// Restarts the computer.
pub fn restart_system() -> bool {
    unsafe {
        ExitWindowsEx(EWX_REBOOT | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0)).is_ok()
    }
}

/// Signs out the current user.
pub fn sign_out_system() -> bool {
    unsafe {
        ExitWindowsEx(EWX_LOGOFF | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0)).is_ok()
    }
}
