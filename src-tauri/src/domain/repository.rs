use crate::domain::hotkey::{Action, Combo, Shortcut};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Неизвестная ошибка: {0}")]
    Internal(String),
}

pub trait ShortcutRepository: Send + Sync {
    fn get(&self, id: &Uuid) -> Result<Option<Shortcut>, RepoError>;
    fn all(&self) -> Result<Vec<Shortcut>, RepoError>;
    fn save(&self, shortcut: &Shortcut) -> Result<(), RepoError>;
    fn delete(&self, id: &Uuid) -> Result<(), RepoError>;
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Невозможно преобразовать")]
    InvalidShortcut,
    #[error("Неизвестная ошибка: {0}")]
    Internal(String),
}

pub trait HotkeyRegistry: Send + Sync {
    fn register(&self, id: &Uuid, combo: &Combo) -> Result<(), RegistryError>;
    fn unregister(&self, id: &Uuid) -> Result<(), RegistryError>;
}

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("Неизвестная ошибка: {0}")]
    Internal(String),
}

pub trait Launcher: Send + Sync {
    fn launch(&self, action: &Action) -> Result<(), LaunchError>;
}
