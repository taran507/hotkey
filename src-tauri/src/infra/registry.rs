use crate::domain::hotkey::{Combo, ShortcutId};
use crate::domain::repository::{HotkeyRegistry, RegistryError};
use crate::infra::tauri_adapter;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub struct TauriHotkeyRegistry {
    app: AppHandle,
    shortcuts: Mutex<HashMap<ShortcutId, Shortcut>>,
}

impl TauriHotkeyRegistry {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            shortcuts: Mutex::new(HashMap::new()),
        }
    }
}

impl HotkeyRegistry for TauriHotkeyRegistry {
    fn register(&self, id: &str, combo: &Combo) -> Result<(), RegistryError> {
        let shortcut =
            tauri_adapter::combo_to_shortcut(combo).ok_or(RegistryError::InvalidShortcut)?;

        self.app
            .global_shortcut()
            .register(shortcut.clone())
            .map_err(|e| RegistryError::Internal(e.to_string()))?;

        self.shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .insert(id.to_string(), shortcut);

        Ok(())
    }

    fn unregister(&self, id: &str) -> Result<(), RegistryError> {
        let shortcut = self
            .shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .get(id)
            .cloned();

        let Some(shortcut) = shortcut else {
            return Ok(());
        };

        self.app
            .global_shortcut()
            .unregister(shortcut)
            .map_err(|e| RegistryError::Internal(e.to_string()))?;

        self.shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .remove(id);

        Ok(())
    }
}
