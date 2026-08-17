use crate::domain::hotkey::{Combo, Mods, PhysicalKey, ShortcutId};
use crate::domain::repository::HotkeyRegistry;
use crate::infra::registry::TauriHotkeyRegistry;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tracing::info;

mod app;
mod desktop;
mod domain;
mod infra;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = mpsc::channel::<ShortcutId>();
    let os_to_shortcut: Arc<Mutex<HashMap<u32, ShortcutId>>> = Arc::new(Mutex::new(HashMap::new()));

    let os_to_shortcut_for_handler = os_to_shortcut.clone();
    let tx_for_handler = tx.clone();

    let plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |_app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }

            let os_id = shortcut.id();

            let Ok(map) = os_to_shortcut_for_handler.lock() else {
                return;
            };

            if let Some(shortcut_id) = map.get(&os_id) {
                let _ = tx_for_handler.send(shortcut_id.clone());
            }
        })
        .build();

    drop(tx);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(plugin)
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let registry = TauriHotkeyRegistry::new(app.handle().clone(), rx, os_to_shortcut);

            if let Err(e) = registry.register(
                "my_id",
                &Combo {
                    key: PhysicalKey("KeyA".to_string()),
                    mods: Mods {
                        ctrl: true,
                        alt: false,
                        logo: false,
                        shift: false,
                    },
                },
            ) {
                println!("registri err {e}")
            };

            let receiver = registry.subscribe();
            std::thread::spawn(move || {
                while let Ok(shortcut_id) = receiver.recv() {
                    println!("Hotkey Press {shortcut_id}");
                }
            });

            app.manage(registry);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
