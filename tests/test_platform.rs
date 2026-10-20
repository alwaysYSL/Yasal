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

#[test]
fn palette_origin_does_not_depend_on_current_window_height() {
    // 1920x1040 work area: the top edge is derived from the tallest palette (500px),
    // so it is identical whether 1 or 8 results are shown.
    let work = (0, 0, 1920, 1040);
    assert_eq!(win32::palette_origin(work, 650, 500), (635, 180));
}

#[test]
fn palette_origin_respects_work_area_offset_and_never_goes_negative() {
    // Taskbar on the left/top shifts the work area.
    assert_eq!(
        win32::palette_origin((48, 40, 1872, 1040), 650, 500),
        (48 + 611, 40 + 180)
    );
    // Window larger than the work area clamps to the top-left corner.
    assert_eq!(win32::palette_origin((0, 0, 400, 300), 650, 500), (0, 0));
}
