use crate::config::{AppConfig, ThemePreference};
use crate::core::dispatcher::Dispatcher;
use crate::core::plugin::QueryResult;
use crate::logging::log as log_app;
use crate::platform::hotkey::HotkeyManager;
use crate::platform::tray::{TrayAction, TrayManager};
use crate::platform::win32;
use crate::plugins::calculator::CalculatorPlugin;
use crate::plugins::clipboard_history::ClipboardHistoryPlugin;
use crate::plugins::file_search::FileSearchPlugin;
use crate::plugins::icon_cache::IconCache;
use crate::plugins::network_info::NetworkInfoPlugin;
use crate::plugins::shell_command::ShellCommandPlugin;
use crate::plugins::system_commands::SystemCommandsPlugin;
use crate::plugins::web_search::WebSearchPlugin;
use crate::plugins::win_settings::WinSettingsPlugin;
use crate::plugins::window_switcher::WindowSwitcherPlugin;
use crate::theme::{
    fade_in_opacity, YasalTheme, CARD_MAX_HEIGHT, CARD_MIN_HEIGHT, CONTENT_HORIZONTAL_MARGIN,
    CONTENT_VERTICAL_MARGIN, FADE_IN_DURATION, MAX_VISIBLE_ITEMS, MAX_WINDOW_HEIGHT, ROW_GAP,
    ROW_HEIGHT, SEARCH_BAR_PANEL_HEIGHT, SHADOW_MARGIN, STATUS_BAR_PANEL_HEIGHT,
    WINDOW_FIXED_HEIGHT, WINDOW_WIDTH,
};
use crate::ui::action_panel::{
    filtered_action_indices, render_filtered_action_panel, ActionPanelEvent,
};
use crate::ui::result_list::{render_result_list, ResultListAction};
use crate::ui::search_bar::{render_search_bar, SearchBarAction, SearchBarState};
use crate::ui::settings_view::{
    filtered_setting_count, next_hotkey_preset, render_settings_view, settings_query_filter,
    toggle_setting_at, SettingsViewEvent, HOTKEY_PRESETS,
};
use crate::ui::status_bar::{render_status_bar, ViewMode};
use eframe::egui;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct YasalApp {
    pub config: AppConfig,
    pub dispatcher: Dispatcher,
    pub hotkey: Option<HotkeyManager>,
    pub tray: Option<TrayManager>,
    pub hwnd: isize,
    pub search_state: SearchBarState,
    pub results: Vec<QueryResult>,
    pub selected_index: usize,
    pub view_mode: ViewMode,
    pub is_visible: bool,
    pub action_panel_item: Option<QueryResult>,
    pub action_panel_index: usize,
    pub settings_index: usize,
    pub icon_cache: IconCache,
    window_had_focus: bool,
    window_height: f32,
    ignore_focus_loss_until: Instant,
    /// Set when the palette is shown; drives the short fade-in animation.
    fade_started: Option<Instant>,

    applied_theme: Option<(bool, (u8, u8, u8))>,
    notice: Option<String>,

    // CPU optimization: cache expensive checks
    last_theme_check: Instant,
    last_focus_check: Instant,
    cached_dark_mode: bool,
    cached_accent: (u8, u8, u8),
}

impl YasalApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: AppConfig,
        dispatcher: Dispatcher,
        hotkey: Option<HotkeyManager>,
        tray: Option<TrayManager>,
        hwnd: isize,
        startup_notice: Option<String>,
    ) -> Self {
        crate::fonts::install(&cc.egui_ctx);

        let now = Instant::now();
        let initial_dark_mode = win32::is_system_dark_mode();
        let initial_accent = win32::get_system_accent_color();

        let mut app = Self {
            config,
            dispatcher,
            hotkey,
            tray,
            hwnd,
            search_state: SearchBarState::default(),
            results: Vec::new(),
            selected_index: 0,
            view_mode: ViewMode::Search,
            is_visible: false, // CPU Fix: Start hidden like a proper command palette
            action_panel_item: None,
            action_panel_index: 0,
            settings_index: 0,
            icon_cache: IconCache::new(),
            window_had_focus: false,
            window_height: 420.0,
            ignore_focus_loss_until: now + Duration::from_millis(180),
            fade_started: None,

            applied_theme: None,
            notice: startup_notice,

            // Initialize cache
            last_theme_check: now,
            last_focus_check: now,
            cached_dark_mode: initial_dark_mode,
            cached_accent: initial_accent,
        };
        app.update_query();
        win32::place_palette_window(
            app.hwnd,
            WINDOW_WIDTH,
            420.0,
            CARD_MAX_HEIGHT,
            SHADOW_MARGIN,
        );

        // CPU Fix: Hide window on startup (command palette should start hidden)
        if app.hwnd != 0 {
            win32::hide_window_deferred(app.hwnd);
        }

        win32::trim_memory_after(std::time::Duration::from_secs(2));
        app
    }

    pub fn toggle_visibility(&mut self, ctx: &egui::Context) {
        self.is_visible = !self.is_visible;
        log_app(&format!(
            "toggle_visibility: is_visible={}",
            self.is_visible
        ));
        if self.is_visible {
            self.window_had_focus = false;
            self.ignore_focus_loss_until = Instant::now() + Duration::from_millis(180);
            self.fade_started = Some(Instant::now());
            self.search_state.text.clear();
            self.search_state.request_focus = true;
            self.view_mode = ViewMode::Search;
            self.update_query();
            // Size and place in one step so the first frame is already the right height,
            // and the top edge is anchored independently of that height.
            if self.hwnd != 0 {
                self.window_height = self.desired_window_height(ctx);
                win32::place_palette_window(
                    self.hwnd,
                    WINDOW_WIDTH,
                    self.window_height,
                    CARD_MAX_HEIGHT,
                    SHADOW_MARGIN,
                );
            }
            if self.hwnd != 0 {
                win32::show_and_focus_window(self.hwnd);
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
            }
        } else {
            self.window_had_focus = false;
            if self.hwnd != 0 {
                win32::hide_window_deferred(self.hwnd);
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                win32::trim_memory();
            }
        }
    }

    pub fn show_window(&mut self, ctx: &egui::Context) {
        if !self.is_visible {
            self.toggle_visibility(ctx);
        } else {
            self.ignore_focus_loss_until = Instant::now() + Duration::from_millis(180);
            win32::place_palette_window(
                self.hwnd,
                WINDOW_WIDTH,
                self.window_height,
                CARD_MAX_HEIGHT,
                SHADOW_MARGIN,
            );
            if self.hwnd != 0 {
                win32::show_and_focus_window(self.hwnd);
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
    }

    pub fn open_settings(&mut self, ctx: &egui::Context) {
        self.show_window(ctx);
        self.view_mode = ViewMode::Settings;
        self.search_state.text.clear();
        self.search_state.request_focus = true;
        self.settings_index = 0;
    }

    pub fn hide_window(&mut self, ctx: &egui::Context) {
        if self.is_visible {
            self.toggle_visibility(ctx);
        }
    }

    fn action_filter(ctx: &egui::Context) -> String {
        ctx.data(|data| {
            data.get_temp::<String>(egui::Id::new("yasal_action_filter"))
                .unwrap_or_default()
        })
    }

    fn update_action_filter(&mut self, ctx: &egui::Context) {
        if let Some(item) = &self.action_panel_item {
            let filter = Self::action_filter(ctx);
            let indices = filtered_action_indices(&item.additional_actions, &filter);
            if !indices.contains(&self.action_panel_index) {
                self.action_panel_index = indices.first().copied().unwrap_or(0);
            }
        }
    }

    pub fn update_query(&mut self) {
        if matches!(self.view_mode, ViewMode::ActionPanel) {
            return;
        }

        if settings_query_filter(&self.search_state.text).is_some() {
            if !matches!(self.view_mode, ViewMode::Settings) {
                self.settings_index = 0;
            }
            self.view_mode = ViewMode::Settings;
            return;
        }

        if matches!(self.view_mode, ViewMode::Settings) {
            self.settings_index = 0;
            return;
        }

        self.view_mode = ViewMode::Search;
        self.results = self.dispatcher.dispatch(&self.search_state.text);
        self.selected_index = 0;
    }

    pub fn execute_selected(&mut self, ctx: &egui::Context) {
        match self.view_mode {
            ViewMode::Search => {
                if let Some(item) = self.results.get(self.selected_index) {
                    let action = item.primary_action.clone();
                    let item_id = item.id.clone();
                    self.dispatcher.record_use(&item_id);
                    self.hide_window(ctx);
                    (action)();
                }
            }
            ViewMode::ActionPanel => {
                if let Some(ref item) = self.action_panel_item {
                    let visible_actions = filtered_action_indices(
                        &item.additional_actions,
                        &Self::action_filter(ctx),
                    );
                    if visible_actions.contains(&self.action_panel_index) {
                        if let Some(action) = item.additional_actions.get(self.action_panel_index) {
                            let exec = action.action.clone();
                            let item_id = item.id.clone();
                            self.dispatcher.record_use(&item_id);
                            self.hide_window(ctx);
                            (exec)();
                        }
                    }
                }
            }
            ViewMode::Settings => self.toggle_setting(self.settings_index),
        }
    }

    pub fn execute_secondary(&mut self, ctx: &egui::Context) {
        if !matches!(self.view_mode, ViewMode::Search) {
            return;
        }
        if let Some(item) = self.results.get(self.selected_index) {
            if let Some(ref sec) = item.secondary_action {
                let action = sec.action.clone();
                let item_id = item.id.clone();
                self.dispatcher.record_use(&item_id);
                self.hide_window(ctx);
                (action)();
            } else {
                let action = item.primary_action.clone();
                let item_id = item.id.clone();
                self.dispatcher.record_use(&item_id);
                self.hide_window(ctx);
                (action)();
            }
        }
    }

    pub fn open_action_panel(&mut self, ctx: &egui::Context) {
        if !matches!(self.view_mode, ViewMode::Search) {
            return;
        }
        if let Some(item) = self.results.get(self.selected_index) {
            if !item.additional_actions.is_empty() {
                ctx.data_mut(|data| {
                    data.remove::<String>(egui::Id::new("yasal_action_filter"));
                });
                self.action_panel_item = Some(item.clone());
                self.action_panel_index = 0;
                self.view_mode = ViewMode::ActionPanel;
                self.search_state.request_focus = true;
            }
        }
    }

    fn sync_plugin(&mut self, id: &str, enabled: bool) {
        self.dispatcher.remove(id);
        if !enabled {
            return;
        }

        match id {
            "calculator" => self.dispatcher.register(Arc::new(CalculatorPlugin::new())),
            "web_search" => self
                .dispatcher
                .register(Arc::new(WebSearchPlugin::new(&self.config.search_engine))),
            "shell_command" => self
                .dispatcher
                .register(Arc::new(ShellCommandPlugin::new())),
            "system_commands" => self
                .dispatcher
                .register(Arc::new(SystemCommandsPlugin::new())),
            "file_search" => self.dispatcher.register(Arc::new(FileSearchPlugin::new())),
            "win_settings" => self.dispatcher.register(Arc::new(WinSettingsPlugin::new())),
            "clipboard_history" => match ClipboardHistoryPlugin::initialize() {
                Ok(plugin) => self.dispatcher.register(Arc::new(plugin)),
                Err(error) => {
                    self.notice = Some(format!("Riwayat clipboard tidak dapat dimulai: {error}"));
                }
            },
            "network_info" => self.dispatcher.register(Arc::new(NetworkInfoPlugin::new())),
            "window_switcher" => self
                .dispatcher
                .register(Arc::new(WindowSwitcherPlugin::new())),
            _ => {}
        }
    }

    /// Applies `config.hotkey` immediately. If another application already owns the
    /// combination, the next preset is tried so cycling never gets stuck on a busy one.
    fn apply_hotkey_change(&mut self) {
        let Some(manager) = self.hotkey.as_ref() else {
            self.notice = Some(format!(
                "Pintasan diubah ke {} dan berlaku setelah Yasal dimulai ulang.",
                self.config.hotkey
            ));
            return;
        };

        let wanted = self.config.hotkey.clone();
        let mut candidate = wanted.clone();
        let mut applied = None;
        for _ in 0..HOTKEY_PRESETS.len() {
            if manager.rebind(&candidate).is_ok() {
                applied = Some(candidate.clone());
                break;
            }
            candidate = next_hotkey_preset(&candidate).to_string();
        }

        match applied {
            Some(active) if active == wanted => {
                self.notice = Some(format!("Pintasan diubah ke {active}."));
            }
            Some(active) => {
                self.notice = Some(format!(
                    "{wanted} sedang digunakan aplikasi lain. Yasal memakai {active}."
                ));
                self.config.hotkey = active;
                if let Err(error) = self.config.save() {
                    self.notice = Some(format!("Konfigurasi tidak dapat disimpan: {error}"));
                }
            }
            None => {
                self.notice = Some(format!(
                    "Pintasan {wanted} tidak dapat dipakai. Pintasan sebelumnya tetap aktif."
                ));
            }
        }
    }

    fn apply_setting_change(&mut self, setting_id: &str) {
        match setting_id {
            "hotkey" => self.apply_hotkey_change(),
            "startup" => {
                if let Err(error) = win32::set_run_on_startup(self.config.run_on_startup) {
                    self.config.run_on_startup = !self.config.run_on_startup;
                    if let Err(save_error) = self.config.save() {
                        self.notice =
                            Some(format!("Konfigurasi tidak dapat disimpan: {save_error}"));
                        return;
                    }
                    self.notice = Some(format!("Pengaturan startup tidak dapat disimpan: {error}"));
                }
            }
            "calc" => self.sync_plugin("calculator", self.config.enable_calculator),
            "web" => self.sync_plugin("web_search", self.config.enable_web_search),
            "shell" => self.sync_plugin("shell_command", self.config.enable_shell_command),
            "sys" => self.sync_plugin("system_commands", self.config.enable_system_commands),
            "engine" => self.sync_plugin("web_search", self.config.enable_web_search),
            "file" => self.sync_plugin("file_search", self.config.enable_file_search),
            "win_settings" => self.sync_plugin("win_settings", self.config.enable_windows_settings),
            "clipboard" => {
                self.sync_plugin("clipboard_history", self.config.enable_clipboard_history)
            }
            "network" => self.sync_plugin("network_info", self.config.enable_network_info),
            "windows" => self.sync_plugin("window_switcher", self.config.enable_window_switcher),
            _ => {}
        }
    }

    fn toggle_setting(&mut self, index: usize) {
        let previous = self.config.clone();
        if let Some(setting_id) =
            toggle_setting_at(&mut self.config, index, &self.search_state.text)
        {
            if let Err(error) = self.config.save() {
                self.config = previous;
                self.notice = Some(format!("Konfigurasi tidak dapat disimpan: {error}"));
                return;
            }
            self.apply_setting_change(setting_id);
        }
    }

    fn desired_window_height(&self, ctx: &egui::Context) -> f32 {
        let row_count = match self.view_mode {
            ViewMode::Search => self.results.len(),
            ViewMode::ActionPanel => self.action_panel_item.as_ref().map_or(0, |item| {
                filtered_action_indices(&item.additional_actions, &Self::action_filter(ctx)).len()
            }),
            ViewMode::Settings => filtered_setting_count(&self.config, &self.search_state.text),
        };
        let content_height = if row_count == 0 {
            120.0
        } else {
            let visible_rows = row_count.min(MAX_VISIBLE_ITEMS);
            visible_rows as f32 * ROW_HEIGHT + visible_rows.saturating_sub(1) as f32 * ROW_GAP
        };
        (WINDOW_FIXED_HEIGHT + content_height + SHADOW_MARGIN * 2.0)
            .clamp(CARD_MIN_HEIGHT + SHADOW_MARGIN * 2.0, MAX_WINDOW_HEIGHT)
    }

    fn sync_window_height(&mut self, ctx: &egui::Context) {
        let desired_height = self.desired_window_height(ctx);
        if (desired_height - self.window_height).abs() > f32::EPSILON {
            self.window_height = desired_height;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                WINDOW_WIDTH,
                desired_height,
            )));
        }
    }

    pub fn handle_escape(&mut self, ctx: &egui::Context) {
        match self.view_mode {
            ViewMode::ActionPanel => {
                ctx.data_mut(|data| {
                    data.remove::<String>(egui::Id::new("yasal_action_filter"));
                });
                self.view_mode = ViewMode::Search;
                self.action_panel_item = None;
                self.search_state.request_focus = true;
            }
            ViewMode::Settings => {
                self.view_mode = ViewMode::Search;
                self.search_state.text.clear();
                self.search_state.request_focus = true;
                self.update_query();
            }
            ViewMode::Search => {
                if !self.search_state.text.is_empty() {
                    self.search_state.text.clear();
                    self.search_state.request_focus = true;
                    self.update_query();
                } else {
                    self.hide_window(ctx);
                }
            }
        }
    }
}

impl eframe::App for YasalApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // CPU Optimization: CRITICAL - Skip expensive operations when window is hidden
        // egui still calls update() even with request_repaint_after(), so we must short-circuit
        if !self.is_visible {
            // Only do minimal polling when hidden
            let hotkey_press_count = self
                .hotkey
                .as_ref()
                .map_or(0, HotkeyManager::poll_press_count);
            for _ in 0..hotkey_press_count {
                log_app("Hotkey pressed event detected (hidden)!");
                self.toggle_visibility(ctx);
            }

            if let Some(ref tray) = self.tray {
                if let Some(action) = tray.poll_action() {
                    match action {
                        TrayAction::ToggleWindow => self.toggle_visibility(ctx),
                        TrayAction::OpenSettings => self.open_settings(ctx),
                        TrayAction::Exit => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            }

            // Hotkey and tray callbacks wake the event loop when needed.
            // A timed repaint on a hidden HWND can leave eframe polling forever.
            return;
        }

        // Poll every queued hotkey event; rapid pairs must not collapse.
        let hotkey_press_count = self
            .hotkey
            .as_ref()
            .map_or(0, HotkeyManager::poll_press_count);
        for _ in 0..hotkey_press_count {
            log_app("Hotkey pressed event detected!");
            self.toggle_visibility(ctx);
        }

        // Poll System Tray
        if let Some(ref tray) = self.tray {
            if let Some(action) = tray.poll_action() {
                match action {
                    TrayAction::ToggleWindow => self.toggle_visibility(ctx),
                    TrayAction::OpenSettings => self.open_settings(ctx),
                    TrayAction::Exit => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        }

        if !self.is_visible {
            return;
        }

        // A command palette should dismiss itself when the user clicks elsewhere.
        // CPU Optimization: Check focus only every 1 second, not every frame
        if self.is_visible {
            let now = Instant::now();
            let should_check_focus =
                now.duration_since(self.last_focus_check) > Duration::from_secs(1);

            if should_check_focus {
                self.last_focus_check = now;
                let viewport_focused = if self.hwnd != 0 {
                    win32::is_foreground_window(self.hwnd)
                } else {
                    ctx.input(|input| input.viewport().focused.unwrap_or(true))
                };

                if viewport_focused {
                    self.window_had_focus = true;
                } else if self.window_had_focus && now >= self.ignore_focus_loss_until {
                    self.hide_window(ctx);
                }
            } else if !self.window_had_focus && now < self.ignore_focus_loss_until {
                // Still in grace period, schedule next check
                ctx.request_repaint_after(self.ignore_focus_loss_until - now);
            }
        }

        let waiting_for_async_results = self.results.iter().any(|result| {
            result.id == "file:indexing"
                || (result.id == "network:status" && result.title.starts_with("Memuat"))
        });
        if self.is_visible
            && matches!(self.view_mode, ViewMode::Search)
            && waiting_for_async_results
        {
            self.results = self.dispatcher.dispatch(&self.search_state.text);
            self.selected_index = self
                .selected_index
                .min(self.results.len().saturating_sub(1));
            if self.results.iter().any(|result| {
                result.id == "file:indexing"
                    || (result.id == "network:status" && result.title.starts_with("Memuat"))
            }) {
                ctx.request_repaint_after(Duration::from_millis(200));
            }
        }

        // CPU Optimization: Cache theme checks - only every 5 seconds, not every frame
        let now = Instant::now();
        if now.duration_since(self.last_theme_check) > Duration::from_secs(5) {
            self.cached_dark_mode = win32::is_system_dark_mode();
            self.cached_accent = win32::get_system_accent_color();
            self.last_theme_check = now;
        }

        // Compute active theme using cached values
        let is_dark = match self.config.theme {
            ThemePreference::Dark => true,
            ThemePreference::Light => false,
            ThemePreference::Auto => self.cached_dark_mode,
        };
        let accent = self.cached_accent;
        let theme = YasalTheme::new(is_dark, accent);
        let theme_signature = (is_dark, accent);
        if self.applied_theme != Some(theme_signature) {
            theme.apply(ctx);
            self.applied_theme = Some(theme_signature);
        }

        // Fade-in: keep repainting for the ~100 ms the animation lasts, then stop.
        let opacity = match self.fade_started {
            Some(started) if started.elapsed() < FADE_IN_DURATION => {
                ctx.request_repaint();
                fade_in_opacity(started.elapsed())
            }
            Some(_) => {
                self.fade_started = None;
                1.0
            }
            None => 1.0,
        };

        theme.paint_window_frame(ctx, opacity);

        // Render the fixed search header and status footer before the scrollable content.
        egui::TopBottomPanel::top("search_bar_panel")
            .exact_height(SEARCH_BAR_PANEL_HEIGHT + SHADOW_MARGIN)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT)
                    .inner_margin(egui::Margin {
                        left: CONTENT_HORIZONTAL_MARGIN + SHADOW_MARGIN,
                        right: CONTENT_HORIZONTAL_MARGIN + SHADOW_MARGIN,
                        top: 12.0 + SHADOW_MARGIN,
                        bottom: 8.0,
                    }),
            )
            .show(ctx, |ui| {
                ui.set_opacity(opacity);
                let mode_icon = match self.view_mode {
                    ViewMode::ActionPanel => "action_panel",
                    ViewMode::Settings => "settings",
                    ViewMode::Search => self.dispatcher.detect_mode_icon(&self.search_state.text),
                };
                let breadcrumb = if let ViewMode::ActionPanel = self.view_mode {
                    self.action_panel_item
                        .as_ref()
                        .map(|item| item.title.as_str())
                } else {
                    None
                };

                let action =
                    render_search_bar(ui, &theme, &mut self.search_state, mode_icon, breadcrumb);

                match action {
                    SearchBarAction::TextChanged => {
                        if matches!(self.view_mode, ViewMode::ActionPanel) {
                            self.update_action_filter(ctx);
                        } else {
                            self.update_query();
                        }
                    }
                    SearchBarAction::Submit => self.execute_selected(ctx),
                    SearchBarAction::SecondaryAction => self.execute_secondary(ctx),
                    SearchBarAction::OpenActionPanel => self.open_action_panel(ctx),
                    SearchBarAction::EscapePressed => self.handle_escape(ctx),
                    SearchBarAction::NavigateUp => match self.view_mode {
                        ViewMode::Search if self.selected_index > 0 => self.selected_index -= 1,
                        ViewMode::ActionPanel => {
                            if let Some(item) = &self.action_panel_item {
                                let indices = filtered_action_indices(
                                    &item.additional_actions,
                                    &Self::action_filter(ctx),
                                );
                                if let Some(position) = indices
                                    .iter()
                                    .position(|&index| index == self.action_panel_index)
                                {
                                    if position > 0 {
                                        self.action_panel_index = indices[position - 1];
                                    }
                                }
                            }
                        }
                        ViewMode::Settings if self.settings_index > 0 => self.settings_index -= 1,
                        _ => {}
                    },
                    SearchBarAction::NavigateDown => match self.view_mode {
                        ViewMode::Search if self.selected_index + 1 < self.results.len() => {
                            self.selected_index += 1
                        }
                        ViewMode::ActionPanel => {
                            if let Some(item) = &self.action_panel_item {
                                let indices = filtered_action_indices(
                                    &item.additional_actions,
                                    &Self::action_filter(ctx),
                                );
                                if let Some(position) = indices
                                    .iter()
                                    .position(|&index| index == self.action_panel_index)
                                {
                                    if position + 1 < indices.len() {
                                        self.action_panel_index = indices[position + 1];
                                    }
                                }
                            }
                        }
                        ViewMode::Settings => {
                            let count =
                                filtered_setting_count(&self.config, &self.search_state.text);
                            if self.settings_index + 1 < count {
                                self.settings_index += 1;
                            }
                        }
                        _ => {}
                    },
                    SearchBarAction::None => {}
                }
            });

        egui::TopBottomPanel::bottom("status_bar_panel")
            .exact_height(STATUS_BAR_PANEL_HEIGHT + SHADOW_MARGIN)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT)
                    .inner_margin(egui::Margin {
                        left: CONTENT_HORIZONTAL_MARGIN + SHADOW_MARGIN,
                        right: CONTENT_HORIZONTAL_MARGIN + SHADOW_MARGIN,
                        top: 0.0,
                        bottom: 8.0 + SHADOW_MARGIN,
                    }),
            )
            .show(ctx, |ui| {
                ui.set_opacity(opacity);
                let selected_category = self
                    .results
                    .get(self.selected_index)
                    .map(|result| result.category.as_str());
                render_status_bar(ui, &theme, &self.view_mode, selected_category);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT)
                    .inner_margin(egui::Margin {
                        left: CONTENT_HORIZONTAL_MARGIN + SHADOW_MARGIN,
                        right: CONTENT_HORIZONTAL_MARGIN + SHADOW_MARGIN,
                        top: CONTENT_VERTICAL_MARGIN,
                        bottom: CONTENT_VERTICAL_MARGIN,
                    }),
            )
            .show(ctx, |ui| {
                ui.set_opacity(opacity);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.view_mode {
                        ViewMode::Search => {
                            if let Some(action) = render_result_list(
                                ui,
                                &theme,
                                &self.results,
                                self.selected_index,
                                &mut self.icon_cache,
                            ) {
                                match action {
                                    ResultListAction::Select(idx) => self.selected_index = idx,
                                    ResultListAction::Execute(idx) => {
                                        self.selected_index = idx;
                                        self.execute_selected(ctx);
                                    }
                                }
                            }
                        }
                        ViewMode::ActionPanel => {
                            if let Some(item) = &self.action_panel_item {
                                if let Some(event) = render_filtered_action_panel(
                                    ui,
                                    &theme,
                                    &item.additional_actions,
                                    &Self::action_filter(ctx),
                                    self.action_panel_index,
                                ) {
                                    match event {
                                        ActionPanelEvent::Select(idx) => {
                                            self.action_panel_index = idx
                                        }
                                        ActionPanelEvent::Execute(idx) => {
                                            self.action_panel_index = idx;
                                            self.execute_selected(ctx);
                                        }
                                        ActionPanelEvent::Close => self.handle_escape(ctx),
                                    }
                                }
                            }
                        }
                        ViewMode::Settings => {
                            if let Some(event) = render_settings_view(
                                ui,
                                &theme,
                                &mut self.config,
                                &self.search_state.text,
                                self.settings_index,
                            ) {
                                match event {
                                    SettingsViewEvent::Select(idx) => self.settings_index = idx,
                                    SettingsViewEvent::Toggle(idx) => {
                                        self.settings_index = idx;
                                        self.toggle_setting(idx);
                                    }
                                    SettingsViewEvent::Close => self.handle_escape(ctx),
                                }
                            }
                        }
                    });
            });

        if let Some(message) = self.notice.clone() {
            let mut dismiss = false;
            egui::Area::new(egui::Id::new("yasal_notice"))
                .anchor(
                    egui::Align2::CENTER_TOP,
                    egui::Vec2::new(0.0, 72.0 + SHADOW_MARGIN),
                )
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(theme.surface_color)
                        .stroke(egui::Stroke::new(1.0_f32, theme.accent_color))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(message).color(theme.text_primary));
                                if ui.small_button("Tutup").clicked() {
                                    dismiss = true;
                                }
                            });
                        });
                });
            if dismiss {
                self.notice = None;
            }
        }

        self.sync_window_height(ctx);

        // CPU Optimization: Conditional repaint based on activity state
        let waiting_for_async_results = self.results.iter().any(|result| {
            result.id == "file:indexing"
                || (result.id == "network:status" && result.title.starts_with("Memuat"))
        });

        if waiting_for_async_results {
            // Active async operations (file indexing, network info) - poll frequently
            ctx.request_repaint_after(Duration::from_millis(200));
        } else {
            // Window visible but idle - repaint occasionally for theme/focus checks
            ctx.request_repaint_after_secs(1.0);
        }

        // Hotkey, tray, pointer, keyboard, and icon completion events
        // wake the event loop explicitly via request_repaint().
    }
}
