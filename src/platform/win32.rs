use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use windows::core::{Error, Result, HSTRING, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, SetLastError, BOOL, HANDLE, HWND, LUID, POINT, RECT, WIN32_ERROR,
};
use windows::Win32::Graphics::Dwm::{
    DwmGetColorizationColor, DwmSetWindowAttribute, DWMWA_BORDER_COLOR,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetMonitorInfoW, InvalidateRect,
    MonitorFromPoint, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::System::Power::SetSuspendState;
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_DWORD, REG_SZ,
};
use windows::Win32::System::Shutdown::{
    ExitWindowsEx, LockWorkStation, EWX_FORCEIFHUNG, EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN,
    SHUTDOWN_REASON,
};
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, GetDpiForWindow, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::Shell::{
    SHEmptyRecycleBinW, SHGetFileInfoW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHFILEINFOW,
    SHGFI_ICON, SHGFI_LARGEICON,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, DestroyIcon, DrawIconEx, GetCursorPos, GetForegroundWindow, GetWindowRect,
    IsWindowVisible, MessageBoxW, SetForegroundWindow, SetWindowPos, ShowWindow,
    SystemParametersInfoW, DI_NORMAL, IDOK, MB_ICONERROR, MB_ICONWARNING, MB_OK, MB_OKCANCEL,
    SPI_GETWORKAREA, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOW, SW_SHOWNOACTIVATE,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

static WINDOW_VISIBILITY_REVISION: AtomicU64 = AtomicU64::new(0);

const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
const TOKEN_QUERY: u32 = 0x0008;
const SE_PRIVILEGE_ENABLED: u32 = 0x0002;
const ERROR_NOT_ALL_ASSIGNED: WIN32_ERROR = WIN32_ERROR(1300);
const SE_SHUTDOWN_NAME_W: &[u16] = &[
    b'S' as u16,
    b'e' as u16,
    b'S' as u16,
    b'h' as u16,
    b'u' as u16,
    b't' as u16,
    b'd' as u16,
    b'o' as u16,
    b'w' as u16,
    b'n' as u16,
    b'P' as u16,
    b'r' as u16,
    b'i' as u16,
    b'v' as u16,
    b'i' as u16,
    b'l' as u16,
    b'e' as u16,
    b'g' as u16,
    b'e' as u16,
    0,
];

#[repr(C)]
struct LuidAndAttributes {
    luid: LUID,
    attributes: u32,
}

#[repr(C)]
struct TokenPrivileges {
    privilege_count: u32,
    privileges: [LuidAndAttributes; 1],
}

// Cargo does not currently enable the windows crate's Win32_Security feature.
// Keep these narrow bindings local so shutdown remains reliable without changing the manifest.
#[link(name = "advapi32")]
extern "system" {
    #[link_name = "OpenProcessToken"]
    fn open_process_token(
        process_handle: HANDLE,
        desired_access: u32,
        token_handle: *mut HANDLE,
    ) -> BOOL;

    #[link_name = "LookupPrivilegeValueW"]
    fn lookup_privilege_value_w(system_name: *const u16, name: *const u16, luid: *mut LUID)
        -> BOOL;

    #[link_name = "AdjustTokenPrivileges"]
    fn adjust_token_privileges(
        token_handle: HANDLE,
        disable_all_privileges: BOOL,
        new_state: *const TokenPrivileges,
        buffer_length: u32,
        previous_state: *mut TokenPrivileges,
        return_length: *mut u32,
    ) -> BOOL;
}

/// Forces Windows to trim the process working set, dropping unused RAM pages.
/// Typical effect: active memory drops from ~20-30 MB down to < 5 MB on window dismiss.
pub fn trim_memory() {
    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
}

/// Trims one time after startup allocations and the first icon textures settle.
pub fn trim_memory_after(delay: std::time::Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        trim_memory();
    });
}

/// Lets the transparent egui surface define the palette's shape.
///
/// The card is drawn by egui with its own corner radius. DWM's rounding (about 8px)
/// and its 1px system border would clip/overdraw that shape and leave stray arcs at
/// the corners, so both are switched off here.
pub fn apply_rounded_corners(hwnd_raw: isize) {
    if hwnd_raw == 0 {
        return;
    }

    unsafe {
        let hwnd = HWND(hwnd_raw as *mut _);

        let preference = DWMWCP_DONOTROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const _,
            std::mem::size_of_val(&preference) as u32,
        );

        // DWMWA_COLOR_NONE: do not draw the system border.
        let no_border: u32 = 0xFFFF_FFFE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &no_border as *const _ as *const _,
            std::mem::size_of_val(&no_border) as u32,
        );
    }
}

/// Adds or removes Yasal from the current user's Windows startup list.
pub fn set_run_on_startup(enabled: bool) -> Result<()> {
    let subkey = HSTRING::from("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = HSTRING::from("Yasal");
    let mut hkey = Default::default();

    unsafe {
        RegOpenKeyExW(HKEY_CURRENT_USER, &subkey, 0, KEY_SET_VALUE, &mut hkey).ok()?;

        let result = if enabled {
            let executable = std::env::current_exe().map_err(|error| {
                Error::new(
                    windows::core::HRESULT(0x80004005u32 as i32),
                    error.to_string(),
                )
            })?;
            let command = format!("\"{}\"", executable.display());
            let utf16: Vec<u16> = command.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes = std::slice::from_raw_parts(
                utf16.as_ptr() as *const u8,
                utf16.len() * std::mem::size_of::<u16>(),
            );
            RegSetValueExW(hkey, &value_name, 0, REG_SZ, Some(bytes)).ok()
        } else {
            let status = RegDeleteValueW(hkey, &value_name);
            if status.is_ok() || status == windows::Win32::Foundation::ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                status.ok()
            }
        };

        let _ = RegCloseKey(hkey);
        result
    }
}

/// Top-left corner of the palette inside a work area, in physical pixels.
///
/// The window is centered horizontally. The vertical position is derived from
/// `anchor_height` (the tallest the palette can get), NOT from the current window
/// height, so the top edge stays at the same place however many results are shown.
/// Using the live height made the palette open at a different Y depending on how tall
/// it happened to be the last time it was used.
pub fn palette_origin(
    work_area: (i32, i32, i32, i32), // left, top, width, height
    window_width: i32,
    anchor_height: i32,
) -> (i32, i32) {
    let (left, top, work_width, work_height) = work_area;
    let x = left + ((work_width - window_width) / 2).max(0);
    let y = top + ((work_height - anchor_height) / 3).max(0);
    (x, y)
}

type MonitorWorkArea = ((i32, i32, i32, i32), Option<u32>);

/// Work area (left, top, width, height) and DPI scale of the monitor under the mouse
/// cursor, i.e. the monitor the user is currently working on.
///
/// Falls back to the primary monitor when the cursor position or monitor info cannot
/// be read. The DPI is `None` if it is unavailable.
fn active_monitor_work_area() -> Option<MonitorWorkArea> {
    unsafe {
        let mut cursor = POINT::default();
        let _ = GetCursorPos(&mut cursor);
        let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTOPRIMARY);

        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return None;
        }

        let (mut dpi_x, mut dpi_y) = (0u32, 0u32);
        let dpi = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y)
            .ok()
            .filter(|_| dpi_x > 0)
            .map(|_| dpi_x);

        let work = info.rcWork;
        Some((
            (
                work.left,
                work.top,
                work.right - work.left,
                work.bottom - work.top,
            ),
            dpi,
        ))
    }
}

/// Places the palette on the active monitor's work area and sizes it in one step.
///
/// `logical_*` values are in egui points; `logical_width/height` describe the whole
/// window including the transparent shadow `margin_logical` on every side, while
/// `anchor_logical_height` is the tallest the visible card can get. The DPI scale comes from the TARGET monitor,
/// not from the window's current size, so moving between monitors with different
/// scaling (e.g. 100% and 125%) still produces the right pixel size. Sizing here
/// (instead of waiting for a later viewport command) avoids a first frame at the
/// previous height.
pub fn place_palette_window(
    hwnd_raw: isize,
    logical_width: f32,
    logical_height: f32,
    anchor_logical_height: f32,
    margin_logical: f32,
) {
    if hwnd_raw == 0 {
        return;
    }

    unsafe {
        let hwnd = HWND(hwnd_raw as *mut _);

        let (work_area, monitor_dpi) = match active_monitor_work_area() {
            Some(found) => found,
            None => {
                // Fallback: primary monitor work area.
                let mut work = RECT::default();
                if SystemParametersInfoW(
                    SPI_GETWORKAREA,
                    0,
                    Some(&mut work as *mut RECT as *mut _),
                    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                )
                .is_err()
                {
                    return;
                }
                (
                    (
                        work.left,
                        work.top,
                        work.right - work.left,
                        work.bottom - work.top,
                    ),
                    None,
                )
            }
        };

        // Prefer the target monitor's DPI; otherwise derive it from the current window.
        let scale = monitor_dpi.map(|dpi| dpi as f32 / 96.0).unwrap_or_else(|| {
            let mut window_rect = RECT::default();
            if GetWindowRect(hwnd, &mut window_rect).is_ok() {
                let current_width = (window_rect.right - window_rect.left) as f32;
                if current_width > 0.0 {
                    return current_width / logical_width;
                }
            }
            1.0
        });

        let width_px = (logical_width * scale).round() as i32;
        let height_px = (logical_height * scale).round() as i32;
        let anchor_px = (anchor_logical_height * scale).round() as i32;
        let margin_px = (margin_logical * scale).round() as i32;

        // Position the visible card (window minus the transparent shadow margin), then
        // move the window out by the margin so the card keeps its exact screen position.
        let (card_x, card_y) = palette_origin(work_area, width_px - margin_px * 2, anchor_px);
        let (x, y) = (card_x - margin_px, card_y - margin_px);

        // Moving a window to a monitor with a different DPI makes Windows rescale its
        // size (WM_DPICHANGED). Setting the size in the same call would be scaled a
        // second time (650px -> 520px or 1016px on a 100%/125% pair). So move first,
        // let the DPI change settle, and only then apply the final size.
        if let Some(dpi) = monitor_dpi {
            if GetDpiForWindow(hwnd) != dpi {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    x,
                    y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }

        let _ = SetWindowPos(
            hwnd,
            None,
            x,
            y,
            width_px,
            height_px,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

/// Extracts an application's native shell icon and stores it as a PNG.
///
/// Shell icon lookup expects COM to be initialized on the calling thread. The icon
/// loader runs this on short-lived worker threads, where it was not, and several
/// lookups in parallel then failed at random (explorer.exe, WindowsTerminal.exe), so a
/// failed attempt is also retried once.
pub fn extract_and_cache_icon(app_path: &Path, out_png_path: &Path) -> bool {
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

    // S_OK and S_FALSE both mean this thread must call CoUninitialize; a failure such
    // as RPC_E_CHANGED_MODE means COM is already usable and must be left alone.
    let com_initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok();

    let mut extracted = extract_icon_once(app_path, out_png_path);
    if !extracted {
        std::thread::sleep(std::time::Duration::from_millis(60));
        extracted = extract_icon_once(app_path, out_png_path);
    }

    if com_initialized {
        unsafe { CoUninitialize() };
    }
    extracted
}

fn extract_icon_once(app_path: &Path, out_png_path: &Path) -> bool {
    const ICON_SIZE: i32 = 64;

    let wide_path: Vec<u16> = app_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut file_info = SHFILEINFOW::default();
        if SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        ) == 0
            || file_info.hIcon.is_invalid()
        {
            return false;
        }

        let memory_dc = CreateCompatibleDC(None);
        if memory_dc.is_invalid() {
            let _ = DestroyIcon(file_info.hIcon);
            return false;
        }

        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: ICON_SIZE,
                biHeight: -ICON_SIZE,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut pixels_ptr = std::ptr::null_mut();
        let bitmap = match CreateDIBSection(
            memory_dc,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut pixels_ptr,
            HANDLE::default(),
            0,
        ) {
            Ok(bitmap) => bitmap,
            Err(_) => {
                let _ = DeleteDC(memory_dc);
                let _ = DestroyIcon(file_info.hIcon);
                return false;
            }
        };

        let previous = SelectObject(memory_dc, bitmap);
        let drawn = DrawIconEx(
            memory_dc,
            0,
            0,
            file_info.hIcon,
            ICON_SIZE,
            ICON_SIZE,
            0,
            None,
            DI_NORMAL,
        )
        .is_ok();

        let byte_len = (ICON_SIZE * ICON_SIZE * 4) as usize;
        let mut rgba = if drawn && !pixels_ptr.is_null() {
            std::slice::from_raw_parts(pixels_ptr as *const u8, byte_len).to_vec()
        } else {
            Vec::new()
        };

        let _ = SelectObject(memory_dc, previous);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(memory_dc);
        let _ = DestroyIcon(file_info.hIcon);

        if rgba.is_empty() {
            return false;
        }

        let has_alpha = rgba.as_chunks::<4>().0.iter().any(|pixel| pixel[3] != 0);
        for pixel in rgba.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
            if !has_alpha && pixel[..3] != [0, 0, 0] {
                pixel[3] = 255;
            }
        }

        if let Some(parent) = out_png_path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return false;
            }
        }

        image::save_buffer(
            out_png_path,
            &rgba,
            ICON_SIZE as u32,
            ICON_SIZE as u32,
            image::ColorType::Rgba8,
        )
        .is_ok()
    }
}

pub fn request_window_redraw(hwnd_raw: isize) {
    if hwnd_raw != 0 {
        unsafe {
            let _ = InvalidateRect(HWND(hwnd_raw as *mut _), None, false);
        }
    }
}

pub fn is_window_visible(hwnd_raw: isize) -> bool {
    hwnd_raw != 0 && unsafe { IsWindowVisible(HWND(hwnd_raw as *mut _)).as_bool() }
}

pub fn is_foreground_window(hwnd_raw: isize) -> bool {
    hwnd_raw != 0 && unsafe { GetForegroundWindow() == HWND(hwnd_raw as *mut _) }
}

/// Activates and brings the specified Win32 window to the foreground.
pub fn show_and_focus_window(hwnd_raw: isize) {
    if hwnd_raw == 0 {
        return;
    }
    WINDOW_VISIBILITY_REVISION.fetch_add(1, Ordering::SeqCst);
    unsafe {
        let hwnd = HWND(hwnd_raw as *mut _);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = BringWindowToTop(hwnd);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(hwnd);
    }
}

/// Makes a hidden window drawable so its event loop can consume a wake-up,
/// without stealing foreground focus from the current application.
pub fn show_window_for_event(hwnd_raw: isize) {
    if hwnd_raw != 0 {
        WINDOW_VISIBILITY_REVISION.fetch_add(1, Ordering::SeqCst);
        unsafe {
            let hwnd = HWND(hwnd_raw as *mut _);
            if !IsWindowVisible(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            }
        }
    }
}

/// Hides after the current eframe redraw has completed. Hiding synchronously
/// inside `App::update` can leave eframe 0.29 polling an invisible HWND.
pub fn hide_window_deferred(hwnd_raw: isize) {
    if hwnd_raw == 0 {
        trim_memory();
        return;
    }

    let revision = WINDOW_VISIBILITY_REVISION.fetch_add(1, Ordering::SeqCst) + 1;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(20));
        if WINDOW_VISIBILITY_REVISION.load(Ordering::SeqCst) == revision {
            hide_window(hwnd_raw);
        }
    });
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

/// Shows an actionable native Windows error message.
pub fn show_error_dialog(title: &str, message: &str) {
    let title_w = HSTRING::from(title);
    let msg_w = HSTRING::from(message);
    unsafe {
        let _ = MessageBoxW(None, &msg_w, &title_w, MB_OK | MB_ICONERROR);
    }
}

/// Empties the Windows Recycle Bin silently.
pub fn empty_recycle_bin() -> bool {
    unsafe { SHEmptyRecycleBinW(None, None, SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI).is_ok() }
}

/// Locks the current Windows workstation.
pub fn lock_workstation() -> bool {
    unsafe { LockWorkStation().is_ok() }
}

/// Puts the system into sleep state.
pub fn sleep_system() -> bool {
    unsafe { SetSuspendState(false, true, false).as_bool() }
}

fn enable_shutdown_privilege() -> Result<()> {
    unsafe {
        let mut token = HANDLE::default();
        if !open_process_token(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        )
        .as_bool()
        {
            return Err(Error::from_win32());
        }

        let result = (|| {
            let mut luid = LUID::default();
            if !lookup_privilege_value_w(std::ptr::null(), SE_SHUTDOWN_NAME_W.as_ptr(), &mut luid)
                .as_bool()
            {
                return Err(Error::from_win32());
            }

            let privileges = TokenPrivileges {
                privilege_count: 1,
                privileges: [LuidAndAttributes {
                    luid,
                    attributes: SE_PRIVILEGE_ENABLED,
                }],
            };

            SetLastError(WIN32_ERROR(0));
            if !adjust_token_privileges(
                token,
                false.into(),
                &privileges,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
            .as_bool()
            {
                return Err(Error::from_win32());
            }

            if GetLastError() == ERROR_NOT_ALL_ASSIGNED {
                return Err(Error::from_win32());
            }

            Ok(())
        })();

        let _ = CloseHandle(token);
        result
    }
}

fn exit_windows_with_shutdown_privilege(
    flags: windows::Win32::System::Shutdown::EXIT_WINDOWS_FLAGS,
) -> Result<()> {
    enable_shutdown_privilege()?;
    unsafe { ExitWindowsEx(flags | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0)) }
}

/// Shuts down the computer and reports the underlying Windows error on failure.
pub fn shutdown_system_result() -> Result<()> {
    exit_windows_with_shutdown_privilege(EWX_SHUTDOWN)
}

/// Shuts down the computer.
pub fn shutdown_system() -> bool {
    shutdown_system_result().is_ok()
}

/// Restarts the computer and reports the underlying Windows error on failure.
pub fn restart_system_result() -> Result<()> {
    exit_windows_with_shutdown_privilege(EWX_REBOOT)
}

/// Restarts the computer.
pub fn restart_system() -> bool {
    restart_system_result().is_ok()
}

/// Signs out the current user and reports the underlying Windows error on failure.
pub fn sign_out_system_result() -> Result<()> {
    unsafe { ExitWindowsEx(EWX_LOGOFF | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0)) }
}

/// Signs out the current user.
pub fn sign_out_system() -> bool {
    sign_out_system_result().is_ok()
}

// ---------------------------------------------------------------------------
// Window switching
// ---------------------------------------------------------------------------

use windows::core::PWSTR;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindow, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, PostMessageW, GWL_EXSTYLE, GW_OWNER, SW_RESTORE, WM_CLOSE,
    WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

/// A top-level window the user can switch to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub exe_path: Option<String>,
    pub minimized: bool,
}

unsafe fn process_image_path(pid: u32) -> Option<String> {
    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
    let mut buffer = [0u16; 1024];
    let mut length = buffer.len() as u32;
    let queried = QueryFullProcessImageNameW(
        process,
        PROCESS_NAME_WIN32,
        PWSTR(buffer.as_mut_ptr()),
        &mut length,
    )
    .is_ok();
    let _ = CloseHandle(process);
    queried.then(|| String::from_utf16_lossy(&buffer[..length as usize]))
}

/// Returns the window if it would appear in Alt+Tab; `None` for tool windows, owned
/// dialogs, cloaked (virtual-desktop / hidden UWP) windows, untitled windows, the
/// desktop shell and Yasal itself.
unsafe fn describe_window(hwnd: HWND) -> Option<WindowInfo> {
    if !IsWindowVisible(hwnd).as_bool() {
        return None;
    }

    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    let is_tool_window = ex_style & WS_EX_TOOLWINDOW.0 != 0;
    let is_app_window = ex_style & WS_EX_APPWINDOW.0 != 0;
    let has_owner = GetWindow(hwnd, GW_OWNER).is_ok_and(|owner| !owner.0.is_null());
    if (is_tool_window || has_owner) && !is_app_window {
        return None;
    }

    let mut cloaked = 0u32;
    if DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut u32 as *mut _,
        std::mem::size_of::<u32>() as u32,
    )
    .is_ok()
        && cloaked != 0
    {
        return None;
    }

    let title_length = GetWindowTextLengthW(hwnd);
    if title_length <= 0 {
        return None;
    }
    let mut buffer = vec![0u16; title_length as usize + 1];
    let copied = GetWindowTextW(hwnd, &mut buffer);
    if copied <= 0 {
        return None;
    }
    let title = String::from_utf16_lossy(&buffer[..copied as usize]);
    if title == "Program Manager" {
        return None;
    }

    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == GetCurrentProcessId() {
        return None;
    }

    Some(WindowInfo {
        hwnd: hwnd.0 as isize,
        title,
        exe_path: process_image_path(pid),
        minimized: IsIconic(hwnd).as_bool(),
    })
}

unsafe extern "system" fn collect_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);
    if let Some(info) = describe_window(hwnd) {
        windows.push(info);
    }
    BOOL(1) // keep enumerating
}

/// Lists switchable top-level windows, topmost (most recently used) first.
pub fn list_top_level_windows() -> Vec<WindowInfo> {
    let mut windows: Vec<WindowInfo> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect_window),
            LPARAM(&mut windows as *mut _ as isize),
        );
    }
    windows
}

/// Brings a window to the foreground, restoring it first if it is minimized.
pub fn focus_window(hwnd_raw: isize) {
    if hwnd_raw == 0 {
        return;
    }
    unsafe {
        let hwnd = HWND(hwnd_raw as *mut _);
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = BringWindowToTop(hwnd);
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Asks a window to close, the same as clicking its close button.
pub fn close_window(hwnd_raw: isize) {
    if hwnd_raw == 0 {
        return;
    }
    unsafe {
        let _ = PostMessageW(HWND(hwnd_raw as *mut _), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
}

/// Shows the native "Select Folder" dialog, owned by the palette window `owner_raw`.
/// Returns `None` when the user cancels or the dialog cannot be created.
pub fn pick_folder(owner_raw: isize) -> Option<std::path::PathBuf> {
    use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{
        FileOpenDialog, IFileOpenDialog, IShellItem, FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS,
        SIGDN_FILESYSPATH,
    };

    unsafe {
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
        let options = dialog.GetOptions().ok()?;
        dialog
            .SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)
            .ok()?;
        let title = HSTRING::from("Pilih folder pencarian file");
        let _ = dialog.SetTitle(&title);

        // Show returns an error when the user cancels, which maps to `None` here.
        dialog.Show(HWND(owner_raw as *mut _)).ok()?;
        let item: IShellItem = dialog.GetResult().ok()?;
        let display = item.GetDisplayName(SIGDN_FILESYSPATH).ok()?;
        let path = display.to_string().ok();
        CoTaskMemFree(Some(display.0 as *const _));
        path.map(std::path::PathBuf::from)
    }
}
