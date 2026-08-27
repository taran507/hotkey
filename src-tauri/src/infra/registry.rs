use crate::domain::hotkey::Combo;
use crate::domain::repository::{HotkeyRegistry, RegistryError};
use crate::infra::tauri_adapter;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use uuid::Uuid;

pub struct TauriHotkeyRegistry {
    app: AppHandle,
    shortcuts: Mutex<HashMap<Uuid, Shortcut>>,
    os_to_shortcut: Mutex<HashMap<u32, Uuid>>,
}

impl TauriHotkeyRegistry {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            shortcuts: Mutex::new(HashMap::new()),
            os_to_shortcut: Mutex::new(HashMap::new()),
        }
    }
}

impl HotkeyRegistry for TauriHotkeyRegistry {
    fn register(&self, id: &Uuid, combo: &Combo) -> Result<(), RegistryError> {
        let shortcut =
            tauri_adapter::combo_to_shortcut(combo).ok_or(RegistryError::InvalidShortcut)?;

        self.app
            .global_shortcut()
            .register(shortcut.clone())
            .map_err(|e| RegistryError::Internal(e.to_string()))?;

        self.shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .insert(id.clone(), shortcut.clone());

        self.os_to_shortcut
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .insert(shortcut.id, id.clone());

        Ok(())
    }

    fn unregister(&self, id: &Uuid) -> Result<(), RegistryError> {
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
            .unregister(shortcut.clone())
            .map_err(|e| RegistryError::Internal(e.to_string()))?;

        self.shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .remove(id);

        self.os_to_shortcut
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .remove(&shortcut.id);

        Ok(())
    }
}
