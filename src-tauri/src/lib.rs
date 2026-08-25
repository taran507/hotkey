use crate::domain::hotkey::{Combo, ShortcutId};
use crate::domain::repository::HotkeyRegistry;
use crate::infra::configs::JsonShortcutRepository;
use crate::infra::launcher::Launcher;
use crate::infra::registry::TauriHotkeyRegistry;
use crate::infra::tauri_adapter;
use crate::interfaces::tauri_command;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use tauri;
use tauri::menu::{Menu, MenuItem};
use tauri::plugin::TauriPlugin;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tracing::{error, info};

mod app;
mod domain;
mod infra;
mod interfaces;

// Настройка сворачивания в трей.
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let Some(icon) = app.default_window_icon().cloned() else {
        return Ok(());
    };

    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    let show_main = |app: &tauri::AppHandle| {
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

fn build_shortcut_plugin() -> (TauriPlugin<tauri::Wry>, mpsc::Receiver<ShortcutId>) {
    let (tx, rx) = mpsc::channel::<ShortcutId>();

    let plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |_app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }

            let Some(combo) = tauri_adapter::shortcut_to_combo(&shortcut) else {
                return;
            };

            let _ = tx.send(combo.id());
        })
        .build();

    (plugin, rx)
}

fn setup_close_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (shortcut_plugin, rx) = build_shortcut_plugin();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(shortcut_plugin)
        .on_window_event(setup_close_event)
        .invoke_handler(tauri::generate_handler![
            tauri_command::list_shortcuts,
            tauri_command::create_shortcut,
            tauri_command::delete_shortcut,
            tauri_command::set_enable_shortcut,
            tauri_command::rename_shortcut
        ])
        .setup(|app| {
            setup_tray(app)?;

            let registry = Arc::new(TauriHotkeyRegistry::new(app.handle().clone()));

            let conf_path = app.path().app_config_dir()?.join("shortcuts.json");
            let repo = Arc::new(
                JsonShortcutRepository::load_or_default(conf_path).map_err(|e| e.to_string())?,
            );
            let launch = Arc::new(Launcher::new());

            let application = app::App::new(repo, registry.clone(), launch);
            let core = Arc::new(application);

            core.register_all_shortcut()?;

            let core_copy = core.clone();

            std::thread::spawn(move || {
                while let Ok(shortcut_id) = rx.recv() {
                    info!("Hotkey Press {shortcut_id}");
                    if let Err(e) = core_copy.run_shortcut(&shortcut_id) {
                        error!("run_shortcut({shortcut_id}): {e}");
                    }
                }
            });

            app.manage(tauri_command::AppState::new(core));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
