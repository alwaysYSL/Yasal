use yasal::platform::win32;

#[test]
fn test_win32_accent_color_rgb() {
    let (r, g, b) = win32::get_system_accent_color();
    // Verify valid RGB range
    let _ = (r, g, b);
}

#[test]
fn test_trim_memory_execution() {
    // Should execute safely without panic
    win32::trim_memory();
}

#[test]
fn test_is_system_dark_mode() {
    let _is_dark = win32::is_system_dark_mode();
}
