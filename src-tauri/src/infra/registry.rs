use crate::domain::hotkey::Combo;
use crate::domain::repository::{HotkeyRegistry, RegistryError};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
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
        let shortcut = combo_to_shortcut(combo).ok_or(RegistryError::InvalidShortcut)?;

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

    fn resolve(&self, os_id: &u32) -> Option<Uuid> {
        self.os_to_shortcut.lock().ok()?.get(os_id).cloned()
    }
}

fn combo_to_shortcut(combo: &Combo) -> Option<Shortcut> {
    let mut modifiers = Modifiers::default();
    if combo.mods.ctrl {
        modifiers |= Modifiers::CONTROL;
    }
    if combo.mods.alt {
        modifiers |= Modifiers::ALT;
    }
    if combo.mods.shift {
        modifiers |= Modifiers::SHIFT;
    }
    if combo.mods.logo {
        modifiers |= Modifiers::SUPER;
    }

    let key = Code::from_str(&combo.key.0).ok()?;

    Some(Shortcut::new(Some(modifiers), key))
}
