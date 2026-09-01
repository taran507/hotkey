use crate::app::App;
use crate::domain::hotkey::{Action, Combo, Shortcut};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::{ShortcutEvent, ShortcutState};
use uuid::Uuid;

#[tauri::command]
pub fn list_shortcuts(state: State<'_, App>) -> Result<Vec<Shortcut>, String> {
    state.list_shortcut().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_shortcut(
    state: State<'_, App>,
    name: String,
    combo: Combo,
    action: Action,
) -> Result<Shortcut, String> {
    state
        .create_shortcut(name, combo, action)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_shortcut(state: State<'_, App>, id: Uuid) -> Result<(), String> {
    state.delete_shortcut(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_shortcut(
    state: State<'_, App>,
    id: Uuid,
    name: String,
    combo: Combo,
    action: Action,
    enabled: bool,
) -> Result<Shortcut, String> {
    state
        .update_shortcut(id, name, combo, action, enabled)
        .map_err(|e| e.to_string())
}

pub fn hotkey_handler(
    app: &AppHandle,
    shortcut: &tauri_plugin_global_shortcut::Shortcut,
    event: ShortcutEvent,
) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let state: State<'_, App> = app.state();
    if let Err(e) = state.run_shortcut(&shortcut.id) {
        log::error!("выполнение шортката: {e}");
    }
}
