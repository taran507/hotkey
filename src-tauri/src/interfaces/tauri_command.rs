use crate::app;
use crate::domain::hotkey::{Action, Combo, Shortcut};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

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
pub fn delete_shortcut(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state.core.delete_shortcut(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_shortcut(
    state: State<'_, AppState>,
    id: Uuid,
    name: String,
    combo: Combo,
    action: Action,
    enabled: bool,
) -> Result<Shortcut, String> {
    state
        .core
        .update_shortcut(id, name, combo, action, enabled)
        .map_err(|e| e.to_string())
}

