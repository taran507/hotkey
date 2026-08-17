use crate::domain::hotkey::{Action, Combo, DomainError, Shortcut};
use crate::domain::repository::{HotkeyRegistry, Launcher, RegistryError, ShortcutRepository};
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum CreateError {
    #[error("невалидное сочетание")]
    InvalidShortcut,
    #[error("сочетание уже существует")]
    AlreadyExist,

    #[error("Неизвестная ошибка: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum EditError {
    #[error("Комбинации не существует")]
    NotFound,

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
        let shortcut = Shortcut::new(combo, action).map_err(|_| CreateError::InvalidShortcut)?;
        if self
            .repo
            .get(&shortcut.id)
            .map_err(|e| CreateError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(CreateError::AlreadyExist);
        }

        let rollback = self
            .register_shortcut(&shortcut)
            .map_err(|e| CreateError::Internal(e.to_string()))?;

        if let Err(e) = self.repo.save(&shortcut) {
            if let Err(e) = rollback() {
                error!("отмена регистрации: {e}")
            }

            return Err(CreateError::Internal(e.to_string()));
        }

        Ok(shortcut)
    }
    pub fn delete_shortcut(&self, id: &str) -> Result<(), EditError> {
        if self
            .repo
            .get(&id)
            .map_err(|_| EditError::Internal(id.to_string()))?
            .is_none()
        {
            return Err(EditError::NotFound);
        }

        self.registry
            .unregister(id)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        self.repo
            .delete(&id)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        Ok(())
    }

    pub fn set_enable_shortcut(&self, id: &str, enabled: bool) -> Result<(), EditError> {
        let mut shortcut = self
            .repo
            .get(&id)
            .map_err(|e| EditError::Internal(e.to_string()))?
            .ok_or(EditError::NotFound)?;

        shortcut.enabled = enabled;

        let rollback = self
            .register_shortcut(&shortcut)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        if let Err(e) = self.repo.save(&shortcut) {
            if let Err(e) = rollback() {
                error!("откат изменений: {e}")
            };
            return Err(EditError::Internal(e.to_string()));
        };

        Ok(())
    }

    fn register_shortcut(
        &self,
        shortcut: &Shortcut,
    ) -> Result<fn() -> Result<(), RegistryError>, RegistryError> {
        if shortcut.enabled {
            self.registry.register(&shortcut.id, &shortcut.combo)?
        } else {
            self.registry.unregister(&shortcut.id)?
        }

        Ok(|| -> Result<(), RegistryError> {
            if shortcut.enabled {
                self.registry.unregister(&shortcut.id)
            } else {
                self.registry.register(&shortcut.id, &shortcut.combo)
            }
        })
    }

    pub fn list_shortcut(&self) -> Result<Vec<Shortcut>, String> {
        self.repo.all().map_err(|e| e.to_string()) // todo(доработать ошибку)
    }

    pub fn register_all_shortcut(&self) -> Result<(), String> {
        let shortcut_list = self.list_shortcut()?;
        let mut rollback_list = Vec::new();

        for shortcut in shortcut_list.iter() {
            // регистрируем хоткей
            match self.register_shortcut(shortcut) {
                Ok(rollback) => {
                    // если успешно зарегистрировали, добавим функцию отката в список
                    rollback_list.push(rollback);
                }
                Err(e) => {
                    // если произошла ошибка, проходимся по списку и откатываем все регистрации.
                    for rollback in rollback_list {
                        if let Err(e) = rollback() {
                            error!("откат транзакции: {e}")
                        }
                    }
                    return Err(e.to_string());
                }
            };
        }

        Ok(())
    }
    pub fn run_shortcut(&self) {}
}
