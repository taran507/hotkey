use crate::domain::hotkey::{Action, Combo, Mods, PhysicalKey};
use crate::infra::configs::JsonShortcutRepository;
use crate::infra::launcher::Launcher;
use crate::infra::registry::TauriHotkeyRegistry;
use crate::infra::resolver;
use crate::interfaces::tauri_command;
use app::App;
use tauri;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log;

mod app;
mod domain;
mod infra;
mod interfaces;

const AUTOSTART_ARG: &str = "--autostart";

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

fn quit_application(app: &tauri::AppHandle) {
    app.exit(0)
}

fn launched_from_autostart() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_ARG)
}

fn setup_plugin(app: &tauri::AppHandle) -> tauri::Result<()> {
    app.plugin(
        tauri_plugin_log::Builder::new()
            .level(log::LevelFilter::Debug)
            .build(),
    )?;
    app.plugin(tauri_plugin_dialog::init())?;
    app.plugin(tauri_plugin_opener::init())?;
    app.plugin(tauri_plugin_autostart::init(
        MacosLauncher::LaunchAgent,
        Some(vec![AUTOSTART_ARG]),
    ))?;
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(tauri_command::hotkey_handler)
            .build(),
    )?;

    Ok(())
}

// Настройка сворачивания в трей.
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
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
            "quit" => quit_application(app),
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                ..
            } => show_main_window(tray.app_handle()),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn setup_core(app: &mut tauri::App) -> Result<(), String> {
    let repo = JsonShortcutRepository::load_or_default(&app)
        .map_err(|e| format!("загрузка конфига: {e}"))?;
    let resolver = resolver::Resolver::new();

    let registry = TauriHotkeyRegistry::new(app.handle().clone(), resolver.clone());
    let launch = Launcher::new();

    let core = App::new(repo, registry, launch, resolver);

    let combo = Combo {
        key: PhysicalKey("KeyR".to_string()),
        mods: Mods {
            ctrl: true,
            alt: true,
            logo: false,
            shift: false,
        },
    };

    let action = Action::Launch {
        program: "".into(),
        args: Vec::new(),
    };

    if let Err(e) = core.create_shortcut("test1".to_string(), combo.clone(), action.clone()) {
        log::error!("create shortcut failed: {e}");
    };

    if let Err(e) = core.create_shortcut("test2".to_string(), combo.clone(), action.clone()) {
        log::error!("create shortcut2 failed: {e}");
    };

    app.manage(core);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            tauri_command::list_shortcuts,
            tauri_command::create_shortcut,
            tauri_command::delete_shortcut,
            tauri_command::update_shortcut,
        ])
        .setup(move |app| {
            setup_plugin(app.handle()).map_err(|e| format!("настройка плагинов: {e}"))?;
            setup_tray(app).map_err(|e| format!("настройка трея: {e}"))?;
            setup_core(app).map_err(|e| format!("настройка ядра приложения: {e}"))?;

            log::debug!("setup");

            if !launched_from_autostart() {
                show_main_window(app.handle());
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| match event {
        RunEvent::Exit => {
            for (_, window) in app.webview_windows() {
                let _ = window.close();
            }
        }
        RunEvent::ExitRequested { api, code, .. } => {
            if code.is_none() {
                api.prevent_exit();
            }
        }
        _ => {}
    });
}
