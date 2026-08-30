use crate::domain::hotkey::{Action, Combo, Shortcut};
use crate::domain::repository::{HotkeyRegistry, Launcher, RegistryError, ShortcutRepository};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

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
    #[error("Невалидная команда")]
    Invalid,
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
        let app = Self {
            repo,
            registry,
            launch,
        };

        app.register_all_shortcut();

        app
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
                log::error!("отмена регистрации: {e}")
            }

            return Err(CreateError::Internal(e.to_string()));
        }

        log::debug!("create shortcut: {:?}", &shortcut);

        Ok(shortcut)
    }

    pub fn delete_shortcut(&self, id: Uuid) -> Result<(), EditError> {
        if self
            .repo
            .get(&id)
            .map_err(|_| EditError::Internal(id.to_string()))?
            .is_none()
        {
            return Err(EditError::NotFound);
        }

        self.registry
            .unregister(&id)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        self.repo
            .delete(&id)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        log::debug!("delete shortcut: {id}");
        Ok(())
    }

    pub fn update_shortcut(
        &self,
        id: Uuid,
        name: String,
        combo: Combo,
        action: Action,
        enabled: bool,
    ) -> Result<Shortcut, EditError> {
        let mut shortcut = self
            .repo
            .get(&id)
            .map_err(|e| EditError::Internal(e.to_string()))?
            .ok_or(EditError::NotFound)?;

        self.registry
            .unregister(&shortcut.id)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        shortcut
            .update(name, combo, action, enabled)
            .map_err(|_| EditError::Invalid)?;

        let rollback = self
            .register_shortcut(&shortcut)
            .map_err(|e| EditError::Internal(e.to_string()))?;

        self.repo.save(&shortcut).map_err(|e| {
            if let Err(e) = rollback() {
                log::error!("отмена регистрации: {e}")
            };
            EditError::Internal(e.to_string())
        })?;

        log::debug!("update shortcut: {:?}", &shortcut);

        Ok(shortcut)
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

        log::debug!("register shortcut: {:?}", &shortcut);

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

    fn register_all_shortcut(&self) {
        let mut shortcut_list = match self.list_shortcut() {
            Ok(list) => list,
            Err(e) => {
                log::error!("получение списка шорткатов: {e}");
                return;
            }
        };

        for shortcut in shortcut_list.iter_mut() {
            if !shortcut.enabled {
                continue;
            }
            // регистрируем хоткей
            if let Err(e) = self.register_shortcut(shortcut) {
                log::error!("регистрация шортката: {e}");
                shortcut.enabled = false;
                if let Err(e) = self.repo.save(&shortcut) {
                    log::error!("сохранение шортката: {e}")
                };
            }
        }
    }

    pub fn run_shortcut(&self, id: &u32) -> Result<(), String> {
        let id = self
            .registry
            .resolve(id)
            .ok_or("Не найдено сочетания клавиш".to_string())?;

        let shortcut = self
            .repo
            .get(&id)
            .map_err(|e| e.to_string())?
            .ok_or("Комбинации не существует".to_string())?;

        if !shortcut.enabled {
            return Ok(());
        }

        self.launch
            .launch(&shortcut.action)
            .map_err(|e| e.to_string())?;

        log::debug!("run shortcut: {:?}", &shortcut);

        Ok(())
    }
}
