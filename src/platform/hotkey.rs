use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
}

impl HotkeyManager {
    /// Creates and registers the default Alt+Space global hotkey.
    pub fn new_alt_space() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;
        let hotkey = HotKey::new(Some(Modifiers::ALT), Code::Space);
        manager.register(hotkey)?;
        Ok(Self { manager, hotkey })
    }

    /// Rebinds the global hotkey with given modifiers and key code.
    pub fn rebind(&mut self, modifiers: Option<Modifiers>, key: Code) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.manager.unregister(self.hotkey);
        let new_hotkey = HotKey::new(modifiers, key);
        self.manager.register(new_hotkey)?;
        self.hotkey = new_hotkey;
        Ok(())
    }

    /// Polls whether the registered hotkey was pressed.
    pub fn poll_is_pressed(&self) -> bool {
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == self.hotkey.id() && event.state == HotKeyState::Pressed {
                return true;
            }
        }
        false
    }
}
