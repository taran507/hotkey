use crate::domain::hotkey::{Combo, Mods, PhysicalKey, ShortcutId};
use crate::domain::repository::{HotkeyRegistry, RegistryError};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use tauri::{App, AppHandle};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

pub struct TauriHotkeyRegistry {
    app: AppHandle,
    rx: Mutex<Option<Receiver<ShortcutId>>>,
    shortcuts: Mutex<HashMap<ShortcutId, Shortcut>>,
    os_to_shortcuts: Arc<Mutex<HashMap<u32, ShortcutId>>>,
}

impl TauriHotkeyRegistry {
    pub fn new(
        app: AppHandle,
        rx: Receiver<ShortcutId>,
        os_to_shortcuts: Arc<Mutex<HashMap<u32, ShortcutId>>>,
    ) -> Self {
        Self {
            app,
            rx: Mutex::new(Some(rx)),
            shortcuts: Mutex::new(HashMap::new()),
            os_to_shortcuts,
        }
    }
}

impl From<&Combo> for Option<Shortcut> {
    fn from(value: &Combo) -> Self {
        let mut modifiers = Modifiers::default();
        if value.mods.ctrl {
            modifiers = modifiers.intersection(Modifiers::CONTROL);
        }
        if value.mods.alt {
            modifiers = modifiers.intersection(Modifiers::ALT);
        }
        if value.mods.shift {
            modifiers = modifiers.intersection(Modifiers::SHIFT);
        }
        if value.mods.logo {
            modifiers = modifiers.intersection(Modifiers::SUPER)
        }

        let key = Code::from_str(&value.key.0).ok()?;

        Some(Shortcut::new(Some(modifiers), key))
    }
}

impl HotkeyRegistry for TauriHotkeyRegistry {
    fn register(&self, id: &str, combo: &Combo) -> Result<(), RegistryError> {
        let o_shortcut: Option<Shortcut> = combo.into();
        let Some(shortcut) = o_shortcut else {
            return Err(RegistryError::Internal(
                "преобразование команды".to_string(),
            ));
        };

        let os_id = shortcut.id();

        self.app
            .global_shortcut()
            .register(shortcut.clone())
            .map_err(|e| RegistryError::Internal(e.to_string()))?;

        self.shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .insert(id.to_string(), shortcut);

        self.os_to_shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .insert(os_id, id.to_string());

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

        let os_id = shortcut.id();

        self.app
            .global_shortcut()
            .unregister(shortcut)
            .map_err(|e| RegistryError::Internal(e.to_string()))?;

        self.shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .remove(id);

        self.os_to_shortcuts
            .lock()
            .map_err(|e| RegistryError::Internal(e.to_string()))?
            .remove(&os_id);

        Ok(())
    }

    fn subscribe(&self) -> Receiver<ShortcutId> {
        self.rx.lock().unwrap().take().unwrap()
    }
}
