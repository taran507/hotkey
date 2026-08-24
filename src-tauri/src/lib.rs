use std::collections::HashMap;
use crate::infra::configs::JsonShortcutRepository;
use crate::infra::launcher::Launcher;
use crate::infra::shortcut_runtime::{build_shortcut_plugin, ShortcutRuntime};
use std::sync::{mpsc, Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager};
use tauri::plugin::TauriPlugin;
use tauri_plugin_global_shortcut::ShortcutState;
use crate::domain::hotkey::ShortcutId;

mod api;
mod app;
mod domain;
mod infra;

// Настройка сворачивания в трей.
fn setup_tray(app: &App) -> tauri::Result<()> {
    let Some(icon) = app.default_window_icon().cloned() else {
        return Ok(());
    };

    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    let show_main = |app: &AppHandle| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    };

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => show_main(app),
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| match event {
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

pub fn build_shortcut_plugin() -> (TauriPlugin<tauri::Wry>, mpsc::Receiver<ShortcutId>, Arc<Mutex<HashMap<u32, ShortcutId>>>) {
    let (tx, rx) = mpsc::channel::<ShortcutId>();
    let os_to_shortcut: Arc<Mutex<HashMap<u32, ShortcutId>>> = Arc::new(Mutex::new(HashMap::new()));

    let os_to_shortcut_for_handler = os_to_shortcut.clone();

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
                let _ = tx.send(shortcut_id.clone());
            }
        })
        .build();

    (plugin, rx, os_to_shortcut)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (shortcut_plugin, rx, os_to_shortcut) = build_shortcut_plugin();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(shortcut_plugin)
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
            api::set_enable_shortcut,
            api::rename_shortcut
        ])
        .setup(|app| {
            let runtime = ShortcutRuntime::new(app.handle().clone(), rx, os_to_shortcut);

            let conf_path = app.path().app_config_dir()?.join("shortcuts.json");
            let repo = Arc::new(
                JsonShortcutRepository::load_or_default(conf_path).map_err(|e| e.to_string())?,
            );
            let launch = Arc::new(Launcher::new());

            let application = app::App::new(repo, runtime.registry(), launch);
            let core = Arc::new(application);

            runtime.register_saved(&core);
            runtime.spawn_listener(core.clone());

            setup_tray(app)?;
            app.manage(api::AppState::new(core));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
