use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
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

        // Generate a 16x16 floral-pink RGBA icon for the tray
        let width = 16;
        let height = 16;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let dx = (x as f32 - 7.5).abs();
                let dy = (y as f32 - 7.5).abs();
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 6.5 {
                    rgba.extend_from_slice(&[255, 105, 180, 255]); // Hot Pink / Cherry blossom
                } else if dist < 7.5 {
                    rgba.extend_from_slice(&[219, 112, 147, 180]); // Soft border
                } else {
                    rgba.extend_from_slice(&[0, 0, 0, 0]); // Transparent
                }
            }
        }

        let icon = Icon::from_rgba(rgba, width, height)?;

        let tray_icon = TrayIconBuilder::new()
            .with_icon(icon)
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
