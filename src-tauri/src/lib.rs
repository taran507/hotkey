use crate::infra::configs::JsonShortcutRepository;
use crate::infra::launcher::Launcher;
use crate::infra::registry::TauriHotkeyRegistry;
use crate::interfaces::tauri_command;
use app::App;
use serde_json::to_string;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{Target, TargetKind};

const AUTOSTART_ARG: &str = "--autostart";
static IS_QUITTING: AtomicBool = AtomicBool::new(false);

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
            "hide" => hide_main_window(app),
            "quit" => quit_app(app),
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

fn quit_app(app: &tauri::AppHandle) {
    IS_QUITTING.store(true, Ordering::SeqCst);
    for (_, window) in app.webview_windows() {
        let _ = window.destroy();
    }
    app.exit(0);
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    match WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("hotkey")
        .inner_size(450.0, 300.0)
        .min_inner_size(400.0, 300.0)
        .always_on_top(false)
        .visible(true)
        .build()
    {
        Ok(window) => {
            let _ = window.set_focus();
        }
        Err(e) => log::error!("failed to create main window: {e}"),
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.destroy();
    }
}

fn launched_from_autostart() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_ARG)
}

fn setup_core(app: &mut tauri::App /*rx: Receiver<u32>*/) -> Result<(), String> {
    let config_path = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("получение папки конфигов: {e}"))?
        .join("shortcuts.json");

    let repo = JsonShortcutRepository::load_or_default(config_path).map_err(|e| e.to_string())?;

    let registry = TauriHotkeyRegistry::new(app.handle().clone());
    let launch = Launcher::new();

    let core = App::new(Arc::new(repo), Arc::new(registry), Arc::new(launch))?;

    app.manage(core);
    Ok(())
}

fn setup_logger(app: &mut tauri::App) {
    // let builder = fmt()
    //     .json()
    //     .with_target(false)
    //     .with_file(true)
    //     .with_line_number(true);
    //
    // let Ok(log_dir) = app.path().app_log_dir() else {
    //     builder.init();
    //     return;
    // };
    //
    // if std::fs::create_dir_all(&log_dir).is_err() {
    //     builder.init();
    //     return;
    // }
    //
    // let log_path = log_dir.join("hotkey.log");
    // let Ok(file) = std::fs::OpenOptions::new()
    //     .create(true)
    //     .append(true)
    //     .open(log_path)
    // else {
    //     builder.init();
    //     return;
    // };
    //
    // builder.with_writer(std::sync::Mutex::new(file)).init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(tauri_command::hotkey_handler)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            tauri_command::list_shortcuts,
            tauri_command::create_shortcut,
            tauri_command::delete_shortcut,
            tauri_command::update_shortcut,
        ])
        .setup(move |app| {
            setup_logger(app);

            if !launched_from_autostart() {
                show_main_window(app.handle());
            }

            setup_tray(app)?;

            setup_core(app)?;
            log::debug!("setup");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            if !IS_QUITTING.load(Ordering::SeqCst) {
                api.prevent_exit();
            }
        }
    });
}
