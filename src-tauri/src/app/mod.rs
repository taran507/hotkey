use crate::domain::hotkey::{Action, Combo, DomainError, Shortcut};
use crate::domain::repository::{HotkeyRegistry, Launcher, ShortcutRepository};
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum CreateError {
    #[error("невалидное сочетание")]
    InvalidSortcut,
    #[error("сочетание уже существует")]
    AlredyExist,

    #[error("Неизвестная ошибка: {0}")]
    Internal(String),
}

pub struct App {
    repo: Arc<dyn ShortcutRepository>,
    registry: Arc<dyn HotkeyRegistry>,
    launch: Arc<dyn Launcher>,
}

impl App {
    pub fn new(
        repo: Arc<dyn ShortcutRepository>,
        registry: Arc<dyn HotkeyRegistry>,
        launch: Arc<dyn Launcher>,
    ) -> Self {
        Self {
            repo,
            registry,
            launch,
        }
    }

    pub fn create_shortcut(&self, combo: Combo, action: Action) -> Result<Shortcut, CreateError> {
        let shortcut = Shortcut::new(combo, action).map_err(|_| CreateError::InvalidSortcut)?;
        if self
            .repo
            .get(&shortcut.id)
            .map_err(|e| CreateError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(CreateError::AlredyExist);
        }

        if shortcut.enabled {
            self.registry
                .register(&shortcut.id, &shortcut.combo)
                .map_err(|e| CreateError::Internal(e.to_string()))?;
        }

        if let Err(e) = self.repo.save(&shortcut) {
            if shortcut.enabled {
                if let Err(e) = self.registry.unregister(&shortcut.id) {
                    error!("отмена регистрации: {e}")
                }
            }
            return Err(CreateError::Internal(e.to_string()));
        }

        Ok(shortcut)
    }
    pub fn delete_shortcut(&self) {}
    pub fn set_enable_shortcut(&self) {}
    pub fn list_shortcut(&self) {}
    pub fn register_all_shortcut(&self) {}
    pub fn run_shortcut(&self) {}
}
