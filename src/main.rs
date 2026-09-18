#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use raw_window_handle::HasWindowHandle;
use std::sync::Arc;
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

fn main() -> Result<(), eframe::Error> {
    let config = AppConfig::load();
    let mut dispatcher = Dispatcher::new();

    // Register active plugins
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

    let hotkey = HotkeyManager::new_alt_space().ok();
    let tray = TrayManager::new().ok();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("🌸 Yasal")
            .with_inner_size([650.0, 420.0])
            .with_min_inner_size([650.0, 100.0])
            .with_max_inner_size([650.0, 420.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Yasal",
        native_options,
        Box::new(move |cc| {
            // Apply Acrylic blur effect if supported on Windows
            #[cfg(target_os = "windows")]
            {
                if let Ok(handle) = cc.window_handle() {
                    let _ = window_vibrancy::apply_acrylic(&handle, Some((20, 20, 20, 200)));
                }
            }

            Ok(Box::new(YasalApp::new(
                cc, config, dispatcher, hotkey, tray,
            )))
        }),
    )
}
