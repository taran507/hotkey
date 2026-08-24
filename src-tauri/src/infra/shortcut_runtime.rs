use crate::app;
use crate::domain::hotkey::ShortcutId;
use crate::domain::repository::HotkeyRegistry;
use crate::infra::registry::TauriHotkeyRegistry;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use tauri::plugin::TauriPlugin;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::ShortcutState;
use tracing::{error, info};


pub struct ShortcutRuntime {
    registry: Arc<TauriHotkeyRegistry>,
}

impl ShortcutRuntime {
    pub fn new(
        app_handle: AppHandle,
        rx: mpsc::Receiver<ShortcutId>,
        os_to_shortcut: Arc<Mutex<HashMap<u32, ShortcutId>>>,
    ) -> Self {
        let registry = Arc::new(TauriHotkeyRegistry::new(app_handle, rx, os_to_shortcut));
        Self { registry }
    }

    pub fn registry(&self) -> Arc<TauriHotkeyRegistry> {
        self.registry.clone()
    }

    pub fn register_saved(&self, core: &app::App) {
        if let Err(e) = core.register_all_shortcut() {
            error!("register_all_shortcut: {e}");
        }
    }

    pub fn spawn_listener(&self, core: Arc<app::App>) {
        let receiver = self.registry.subscribe();

        std::thread::spawn(move || {
            while let Ok(shortcut_id) = receiver.recv() {
                info!("Hotkey Press {shortcut_id}");
                if let Err(e) = core.run_shortcut(&shortcut_id) {
                    error!("run_shortcut({shortcut_id}): {e}");
                }
            }
        });
    }
}

