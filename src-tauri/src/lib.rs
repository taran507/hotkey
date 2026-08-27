use crate::infra::configs::JsonShortcutRepository;
use crate::infra::launcher::Launcher;
use crate::infra::registry::TauriHotkeyRegistry;
use crate::interfaces::{tauri_command, tauri_hotkey};
use std::sync::{mpsc, Arc};
use tauri;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

const AUTOSTART_ARG: &str = "--autostart";

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

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
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
            } => show_main_window(tray.app_handle()),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn launched_from_autostart() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_ARG)
}

fn setup_close_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = mpsc::channel::<u32>();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(tauri_hotkey::hotkey_handler(tx))
                .build(),
        )
        .on_window_event(setup_close_event)
        .invoke_handler(tauri::generate_handler![
            tauri_command::list_shortcuts,
            tauri_command::create_shortcut,
            tauri_command::delete_shortcut,
            tauri_command::update_shortcut,
        ])
        .setup(move |app| {
            setup_tray(app)?;
            if !launched_from_autostart() {
                show_main_window(app.handle());
            }

            let registry = Arc::new(TauriHotkeyRegistry::new(app.handle().clone()));

            let conf_path = app.path().app_config_dir()?.join("shortcuts.json");
            let repo = Arc::new(
                JsonShortcutRepository::load_or_default(conf_path).map_err(|e| e.to_string())?,
            );

            let launch = Arc::new(Launcher::new());
            let application = app::App::new(repo, registry.clone(), launch)?;

            let core = Arc::new(application);

            app.manage(tauri_command::AppState::new(core.clone()));

            tauri_hotkey::spawn_worker(rx, core.clone(), registry);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
