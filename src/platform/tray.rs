use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TrayAction {
    ToggleWindow,
    OpenSettings,
    Exit,
}

pub struct TrayManager {
    _tray_icon: TrayIcon,
    show_item: MenuItem,
    settings_item: MenuItem,
    exit_item: MenuItem,
}

impl TrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();
        let show_item = MenuItem::new("Show Yasal (Alt+Space)", true, None);
        let settings_item = MenuItem::new("Settings", true, None);
        let exit_item = MenuItem::new("Exit", true, None);

        menu.append_items(&[&show_item, &settings_item, &exit_item])?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("🌸 Yasal — Command Palette")
            .build()?;

        Ok(Self {
            _tray_icon: tray_icon,
            show_item,
            settings_item,
            exit_item,
        })
    }

    pub fn poll_action(&self) -> Option<TrayAction> {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.show_item.id() {
                return Some(TrayAction::ToggleWindow);
            } else if event.id == self.settings_item.id() {
                return Some(TrayAction::OpenSettings);
            } else if event.id == self.exit_item.id() {
                return Some(TrayAction::Exit);
            }
        }
        None
    }
}
