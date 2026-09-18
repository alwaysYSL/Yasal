use crate::config::{AppConfig, ThemePreference};
use crate::core::dispatcher::Dispatcher;
use crate::core::plugin::QueryResult;
use crate::platform::hotkey::HotkeyManager;
use crate::platform::tray::{TrayAction, TrayManager};
use crate::platform::win32;
use crate::theme::YasalTheme;
use crate::ui::action_panel::{render_action_panel, ActionPanelEvent};
use crate::ui::result_list::{render_result_list, ResultListAction};
use crate::ui::search_bar::{render_search_bar, SearchBarAction, SearchBarState};
use crate::ui::settings_view::{render_settings_view, toggle_setting_at, SettingsViewEvent};
use crate::ui::status_bar::{render_status_bar, ViewMode};
use eframe::egui;

pub struct YasalApp {
    pub config: AppConfig,
    pub dispatcher: Dispatcher,
    pub hotkey: Option<HotkeyManager>,
    pub tray: Option<TrayManager>,
    pub search_state: SearchBarState,
    pub results: Vec<QueryResult>,
    pub selected_index: usize,
    pub view_mode: ViewMode,
    pub is_visible: bool,
    pub action_panel_item: Option<QueryResult>,
    pub action_panel_index: usize,
    pub settings_index: usize,
}

impl YasalApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        config: AppConfig,
        dispatcher: Dispatcher,
        hotkey: Option<HotkeyManager>,
        tray: Option<TrayManager>,
    ) -> Self {
        let mut app = Self {
            config,
            dispatcher,
            hotkey,
            tray,
            search_state: SearchBarState::default(),
            results: Vec::new(),
            selected_index: 0,
            view_mode: ViewMode::Search,
            is_visible: true,
            action_panel_item: None,
            action_panel_index: 0,
            settings_index: 0,
        };
        app.update_query();
        app
    }

    pub fn toggle_visibility(&mut self, ctx: &egui::Context) {
        self.is_visible = !self.is_visible;
        if self.is_visible {
            // Fresh input reset when opening
            self.search_state.text.clear();
            self.search_state.request_focus = true;
            self.view_mode = ViewMode::Search;
            self.update_query();
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            // Memory trimming on window hide -> RAM drops to < 5 MB
            win32::trim_memory();
        }
    }

    pub fn show_window(&mut self, ctx: &egui::Context) {
        if !self.is_visible {
            self.toggle_visibility(ctx);
        }
    }

    pub fn hide_window(&mut self, ctx: &egui::Context) {
        if self.is_visible {
            self.toggle_visibility(ctx);
        }
    }

    pub fn update_query(&mut self) {
        if self.search_state.text.trim().eq_ignore_ascii_case("settings") {
            self.view_mode = ViewMode::Settings;
            self.settings_index = 0;
            return;
        }

        if matches!(self.view_mode, ViewMode::Settings) && !self.search_state.text.trim().is_empty() {
            // In settings filtering mode
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
                    if let Some(action) = item.additional_actions.get(self.action_panel_index) {
                        let exec = action.action.clone();
                        let item_id = item.id.clone();
                        self.dispatcher.record_use(&item_id);
                        self.hide_window(ctx);
                        (exec)();
                    }
                }
            }
            ViewMode::Settings => {
                toggle_setting_at(&mut self.config, self.settings_index, &self.search_state.text);
            }
        }
    }

    pub fn execute_secondary(&mut self, ctx: &egui::Context) {
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

    pub fn open_action_panel(&mut self) {
        if let Some(item) = self.results.get(self.selected_index) {
            if !item.additional_actions.is_empty() {
                self.action_panel_item = Some(item.clone());
                self.action_panel_index = 0;
                self.view_mode = ViewMode::ActionPanel;
            }
        }
    }

    pub fn handle_escape(&mut self, ctx: &egui::Context) {
        match self.view_mode {
            ViewMode::ActionPanel => {
                self.view_mode = ViewMode::Search;
                self.action_panel_item = None;
            }
            ViewMode::Settings => {
                self.view_mode = ViewMode::Search;
                self.search_state.text.clear();
                self.update_query();
            }
            ViewMode::Search => {
                // 2-Step Escape: If text is present, clear text; otherwise close window
                if !self.search_state.text.is_empty() {
                    self.search_state.text.clear();
                    self.update_query();
                } else {
                    self.hide_window(ctx);
                }
            }
        }
    }
}

impl eframe::App for YasalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll Global Hotkey
        if let Some(ref hotkey) = self.hotkey {
            if hotkey.poll_is_pressed() {
                self.toggle_visibility(ctx);
            }
        }

        // Poll System Tray
        if let Some(ref tray) = self.tray {
            if let Some(action) = tray.poll_action() {
                match action {
                    TrayAction::ToggleWindow => self.toggle_visibility(ctx),
                    TrayAction::OpenSettings => {
                        self.show_window(ctx);
                        self.view_mode = ViewMode::Settings;
                        self.search_state.text.clear();
                    }
                    TrayAction::Exit => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        }

        // Compute active theme
        let is_dark = match self.config.theme {
            ThemePreference::Dark => true,
            ThemePreference::Light => false,
            ThemePreference::Auto => win32::is_system_dark_mode(),
        };
        let accent = win32::get_system_accent_color();
        let theme = YasalTheme::new(is_dark, accent);
        theme.apply(ctx);

        // Render Command Palette Window
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 8.0);

            // 1. Search Bar
            let mode_icon = match self.view_mode {
                ViewMode::ActionPanel => "action_panel",
                ViewMode::Settings => "settings",
                ViewMode::Search => self.dispatcher.detect_mode_icon(&self.search_state.text),
            };

            let breadcrumb = if let ViewMode::ActionPanel = self.view_mode {
                self.action_panel_item.as_ref().map(|i| i.title.as_str())
            } else {
                None
            };

            let sb_action = render_search_bar(ui, &theme, &mut self.search_state, mode_icon, breadcrumb);

            match sb_action {
                SearchBarAction::TextChanged => self.update_query(),
                SearchBarAction::Submit => self.execute_selected(ctx),
                SearchBarAction::SecondaryAction => self.execute_secondary(ctx),
                SearchBarAction::OpenActionPanel => self.open_action_panel(),
                SearchBarAction::EscapePressed => self.handle_escape(ctx),
                SearchBarAction::NavigateUp => match self.view_mode {
                    ViewMode::Search => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                    }
                    ViewMode::ActionPanel => {
                        if self.action_panel_index > 0 {
                            self.action_panel_index -= 1;
                        }
                    }
                    ViewMode::Settings => {
                        if self.settings_index > 0 {
                            self.settings_index -= 1;
                        }
                    }
                },
                SearchBarAction::NavigateDown => match self.view_mode {
                    ViewMode::Search => {
                        if !self.results.is_empty() && self.selected_index + 1 < self.results.len() {
                            self.selected_index += 1;
                        }
                    }
                    ViewMode::ActionPanel => {
                        if let Some(ref item) = self.action_panel_item {
                            if self.action_panel_index + 1 < item.additional_actions.len() {
                                self.action_panel_index += 1;
                            }
                        }
                    }
                    ViewMode::Settings => {
                        self.settings_index += 1;
                    }
                },
                SearchBarAction::None => {}
            }

            // 2. Results / Content Area
            match self.view_mode {
                ViewMode::Search => {
                    if let Some(action) = render_result_list(ui, &theme, &self.results, self.selected_index) {
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
                    if let Some(ref item) = self.action_panel_item {
                        if let Some(event) = render_action_panel(
                            ui,
                            &theme,
                            &item.additional_actions,
                            self.action_panel_index,
                        ) {
                            match event {
                                ActionPanelEvent::Select(idx) => self.action_panel_index = idx,
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
                                toggle_setting_at(&mut self.config, idx, &self.search_state.text);
                            }
                            SettingsViewEvent::Close => self.handle_escape(ctx),
                        }
                    }
                }
            }

            // 3. Status Bar
            let selected_category = self.results.get(self.selected_index).map(|r| r.category.as_str());
            render_status_bar(ui, &theme, &self.view_mode, selected_category);
        });

        // Request low refresh rate while idling to keep CPU at ~0%
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
