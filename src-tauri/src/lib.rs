use crate::domain::hotkey::ShortcutId;
use crate::domain::repository::{HotkeyRegistry, ShortcutRepository};
use crate::infra::configs::JsonShortcutRepository;
use crate::infra::launcher::Launcher;
use crate::infra::registry::TauriHotkeyRegistry;
use std::collections::HashMap;
use std::io;
use std::sync::{mpsc, Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager};
use tauri_plugin_global_shortcut::ShortcutState;
use tracing::{error, info};

mod api;
mod app;
mod domain;
mod infra;

fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn setup_tray(app: &App) -> tauri::Result<()> {
    let Some(icon) = app.default_window_icon().cloned() else {
        return Ok(());
    };

    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main(app),
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => show_main(tray.app_handle()),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn spawn_shortcut_listener(registry: Arc<dyn HotkeyRegistry>, core: Arc<app::App>) {
    let receiver = registry.subscribe();
    std::thread::spawn(move || {
        while let Ok(shortcut_id) = receiver.recv() {
            info!("Hotkey Press {shortcut_id}");
            if let Err(e) = core.run_shortcut(&shortcut_id) {
                error!("run_shortcut({shortcut_id}): {e}");
            }
        }
    });
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
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            api::list_shortcuts,
            api::create_shortcut,
            api::delete_shortcut,
            api::set_enable_shortcut
        ])
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?.join("shortcuts.json");

            let storage: Arc<dyn ShortcutRepository> =
                Arc::new(JsonShortcutRepository::load_or_default(config_dir).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, e.to_string())
                })?);

            let registry = Arc::new(TauriHotkeyRegistry::new(
                app.handle().clone(),
                rx,
                os_to_shortcut,
            ));

            let launcher = Arc::new(Launcher::new());
            let core = Arc::new(app::App::new(storage, registry.clone(), launcher));

            // Register all enabled shortcuts from persisted config.
            if let Err(e) = core.register_all_shortcut() {
                error!("register_all_shortcut: {e}");
            }

            spawn_shortcut_listener(registry.clone(), core.clone());

            setup_tray(app)?;
            app.manage(api::AppState::new(core));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
