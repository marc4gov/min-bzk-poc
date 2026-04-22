use crate::errors::Result;

pub struct HotkeyService {
    registered: bool,
}

impl HotkeyService {
    pub fn new() -> Self {
        Self { registered: false }
    }

    pub fn is_registered(&self) -> bool {
        self.registered
    }

    // Placeholder: Will be implemented with tauri-plugin-global-shortcut
    pub async fn register(&mut self, _hotkey: &str) -> Result<()> {
        // TODO: Implement with tauri-plugin-global-shortcut
        self.registered = true;
        tracing::info!("Hotkey registered (placeholder)");
        Ok(())
    }

    pub async fn unregister(&mut self) -> Result<()> {
        self.registered = false;
        tracing::info!("Hotkey unregistered (placeholder)");
        Ok(())
    }
}

impl Default for HotkeyService {
    fn default() -> Self {
        Self::new()
    }
}
