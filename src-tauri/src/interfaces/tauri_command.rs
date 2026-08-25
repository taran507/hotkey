use crate::app;
use crate::domain::hotkey::{Action, Combo, Shortcut};
use std::sync::Arc;
use tauri::State;

#[derive(Clone)]
pub struct AppState {
    core: Arc<app::App>,
}

impl AppState {
    pub fn new(core: Arc<app::App>) -> Self {
        Self { core }
    }
}

#[tauri::command]
pub fn list_shortcuts(state: State<'_, AppState>) -> Result<Vec<Shortcut>, String> {
    state.core.list_shortcut()
}

#[tauri::command]
pub fn create_shortcut(
    state: State<'_, AppState>,
    name: String,
    combo: Combo,
    action: Action,
) -> Result<Shortcut, String> {
    state
        .core
        .create_shortcut(name, combo, action)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_shortcut(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.core.delete_shortcut(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_enable_shortcut(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    state
        .core
        .set_enable_shortcut(&id, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_shortcut(state: State<'_, AppState>, id: String, name: String) -> Result<(), String> {
    state
        .core
        .rename_shortcut(&id, name)
        .map_err(|e| e.to_string())
}
