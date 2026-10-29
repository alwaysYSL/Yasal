#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;
use raw_window_handle::HasWindowHandle;
use std::sync::Arc;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use yasal::app::YasalApp;
use yasal::config::AppConfig;
use yasal::core::dispatcher::Dispatcher;
use yasal::logging::log as log_msg;
use yasal::platform::hotkey::HotkeyManager;
use yasal::platform::tray::TrayManager;
use yasal::plugins::app_launcher::AppLauncherPlugin;
use yasal::plugins::calculator::CalculatorPlugin;
use yasal::plugins::clipboard_history::ClipboardHistoryPlugin;
use yasal::plugins::file_search::FileSearchPlugin;
use yasal::plugins::network_info::NetworkInfoPlugin;
use yasal::plugins::shell_command::ShellCommandPlugin;
use yasal::plugins::system_commands::SystemCommandsPlugin;
use yasal::plugins::web_search::WebSearchPlugin;
use yasal::plugins::win_settings::WinSettingsPlugin;
use yasal::plugins::window_switcher::WindowSwitcherPlugin;
use yasal::theme::{MAX_WINDOW_HEIGHT, WINDOW_WIDTH};

fn main() {
    log_msg("=== Yasal Starting ===");

    // Set panic hook to log any panics to file
    std::panic::set_hook(Box::new(|info| {
        log_msg(&format!("PANIC: {:?}", info));
    }));

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }

    let config = AppConfig::load();
    let mut dispatcher = Dispatcher::new();

    let app_launcher = Arc::new(AppLauncherPlugin::new());
    dispatcher.register(app_launcher);

    if config.enable_calculator {
        dispatcher.register(Arc::new(CalculatorPlugin::new()));
    }

    if config.enable_web_search {
        dispatcher.register(Arc::new(WebSearchPlugin::with_aliases(
            &config.search_engine,
            &config.web_aliases,
        )));
    }

    if config.enable_shell_command {
        dispatcher.register(Arc::new(ShellCommandPlugin::new()));
    }

    if config.enable_system_commands {
        dispatcher.register(Arc::new(SystemCommandsPlugin::new()));
    }

    if config.enable_file_search {
        dispatcher.register(Arc::new(FileSearchPlugin::with_folders(
            config.search_folder_paths(),
        )));
    }

    if config.enable_windows_settings {
        dispatcher.register(Arc::new(WinSettingsPlugin::new()));
    }

    if config.enable_clipboard_history {
        match ClipboardHistoryPlugin::initialize() {
            Ok(plugin) => dispatcher.register(Arc::new(plugin)),
            Err(error) => log_msg(&format!("Clipboard history initialization failed: {error}")),
        }
    }

    if config.enable_window_switcher {
        dispatcher.register(Arc::new(WindowSwitcherPlugin::new()));
    }

    if config.enable_network_info {
        dispatcher.register(Arc::new(NetworkInfoPlugin::new()));
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ðŸŒ¸ Yasal")
            .with_inner_size([WINDOW_WIDTH, 420.0])
            .with_min_inner_size([WINDOW_WIDTH, 100.0])
            .with_max_inner_size([WINDOW_WIDTH, MAX_WINDOW_HEIGHT])
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_active(true),

        // CPU Optimization: Set max FPS to reduce continuous rendering
        // This doesn't fully work in older egui versions, but helps
        vsync: true,

        ..Default::default()
    };

    log_msg("Calling eframe::run_native...");

    if let Err(err) = eframe::run_native(
        "Yasal",
        native_options,
        Box::new(move |cc| {
            log_msg("eframe creation context initialized");

            let mut window_hwnd: isize = 0;
            #[cfg(target_os = "windows")]
            {
                if let Ok(handle) = cc.window_handle() {
                    if let raw_window_handle::RawWindowHandle::Win32(win32_handle) = handle.as_raw()
                    {
                        window_hwnd = win32_handle.hwnd.get();
                        HotkeyManager::set_window_hwnd(window_hwnd);
                        yasal::platform::win32::apply_rounded_corners(window_hwnd);
                        log_msg(&format!("Window HWND acquired: 0x{:X}", window_hwnd));
                    }
                }
            }

            let (hotkey, mut startup_notice) =
                match HotkeyManager::new(cc.egui_ctx.clone(), &config.hotkey) {
                    Ok(manager) => {
                        let warning = manager.warning().map(str::to_owned);
                        (Some(manager), warning)
                    }
                    Err(error) => {
                        let message = format!("Pintasan global tidak aktif: {error}");
                        log_msg(&message);
                        (None, Some(message))
                    }
                };
            let tray = match TrayManager::new(cc.egui_ctx.clone(), window_hwnd) {
                Ok(manager) => Some(manager),
                Err(error) => {
                    let message = format!("Tray sistem tidak dapat dibuat: {error}");
                    log_msg(&message);
                    startup_notice = Some(message);
                    None
                }
            };

            log_msg("YasalApp created successfully");

            Ok(Box::new(YasalApp::new(
                cc,
                config,
                dispatcher,
                hotkey,
                tray,
                window_hwnd,
                startup_notice,
            )))
        }),
    ) {
        log_msg(&format!("eframe::run_native error: {:?}", err));
    }

    log_msg("=== Yasal Exited ===");
}
