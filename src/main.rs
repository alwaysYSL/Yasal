use eframe::egui;
use raw_window_handle::HasWindowHandle;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use yasal::app::YasalApp;
use yasal::config::AppConfig;
use yasal::core::dispatcher::Dispatcher;
use yasal::platform::hotkey::HotkeyManager;
use yasal::platform::tray::TrayManager;
use yasal::plugins::app_launcher::AppLauncherPlugin;
use yasal::plugins::calculator::CalculatorPlugin;
use yasal::plugins::shell_command::ShellCommandPlugin;
use yasal::plugins::system_commands::SystemCommandsPlugin;
use yasal::plugins::web_search::WebSearchPlugin;

fn log_msg(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("yasal_debug.log")
    {
        let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S%.3f"), msg);
    }
}

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
        dispatcher.register(Arc::new(WebSearchPlugin::new(&config.search_engine)));
    }

    if config.enable_shell_command {
        dispatcher.register(Arc::new(ShellCommandPlugin::new()));
    }

    if config.enable_system_commands {
        dispatcher.register(Arc::new(SystemCommandsPlugin::new()));
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("🌸 Yasal")
            .with_inner_size([650.0, 420.0])
            .with_min_inner_size([650.0, 100.0])
            .with_max_inner_size([650.0, 420.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_always_on_top()
            .with_active(true),
        ..Default::default()
    };

    log_msg("Calling eframe::run_native...");

    if let Err(err) = eframe::run_native(
        "Yasal",
        native_options,
        Box::new(move |cc| {
            log_msg("eframe creation context initialized");

            #[cfg(target_os = "windows")]
            {
                if let Ok(handle) = cc.window_handle() {
                    let _ = window_vibrancy::apply_acrylic(&handle, Some((20, 20, 20, 200)));
                }
            }

            let hotkey = HotkeyManager::new(cc.egui_ctx.clone()).ok();
            let tray = TrayManager::new().ok();

            log_msg("YasalApp created successfully");

            Ok(Box::new(YasalApp::new(
                cc, config, dispatcher, hotkey, tray,
            )))
        }),
    ) {
        log_msg(&format!("eframe::run_native error: {:?}", err));
    }

    log_msg("=== Yasal Exited ===");
}
