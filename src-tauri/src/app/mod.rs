use crate::domain::hotkey::{Action, Combo, Shortcut};
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

    pub fn create_shortcut(
        &self,
        name: String,
        combo: Combo,
        action: Action,
    ) -> Result<Shortcut, CreateError> {
        let shortcut =
            Shortcut::new(name, combo, action).map_err(|_| CreateError::InvalidShortcut)?;
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

    pub fn rename_shortcut(&self, id: &str, name: String) -> Result<(), EditError> {
        let mut shortcut = self
            .repo
            .get(id)
            .map_err(|e| EditError::Internal(e.to_string()))?
            .ok_or(EditError::NotFound)?;

        shortcut
            .rename(name)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        self.repo
            .save(&shortcut)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        Ok(())
    }

    fn register_shortcut(
        &self,
        shortcut: &Shortcut,
    ) -> Result<Box<dyn FnOnce() -> Result<(), RegistryError>>, RegistryError> {
        let id = shortcut.id.clone();
        let combo = shortcut.combo.clone();
        let enabled = shortcut.enabled;
        let registry = Arc::clone(&self.registry);

        if enabled {
            registry.register(&id, &combo)?
        } else {
            registry.unregister(&id)?
        }

        Ok(Box::new(move || -> Result<(), RegistryError> {
            if enabled {
                registry.unregister(&id)
            } else {
                registry.register(&id, &combo)
            }
        }))
    }

    pub fn list_shortcut(&self) -> Result<Vec<Shortcut>, String> {
        self.repo.all().map_err(|e| e.to_string()) // todo(доработать ошибку)
    }

    pub fn register_all_shortcut(&self) -> Result<(), String> {
        let shortcut_list = self.list_shortcut()?;
        let mut rollback_list = Vec::new();

        for shortcut in shortcut_list.iter() {
            if !shortcut.enabled {
                continue;
            }
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
    pub fn run_shortcut(&self, id: &str) -> Result<(), String> {
        let shortcut = self
            .repo
            .get(id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Комбинации не существует".to_string())?;

        if !shortcut.enabled {
            return Ok(());
        }

        self.launch.launch(&shortcut.action).map_err(|e| e.to_string())
    }
}
