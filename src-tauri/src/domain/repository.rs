use crate::domain::hotkey::{Action, Combo, Shortcut, ShortcutId};
use std::sync::mpsc::Receiver;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Неизвестная ошибка: {0}")]
    Internal(String),
}

pub trait ShortcutRepository: Send + Sync {
    fn get(&self, id: &str) -> Result<Option<Shortcut>, RepoError>;
    fn all(&self) -> Result<Vec<Shortcut>, RepoError>;
    fn save(&self, shortcut: &Shortcut) -> Result<(), RepoError>;
    fn delete(&self, id: &str) -> Result<(), RepoError>;
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Неизвестная ошибка: {0}")]
    Internal(String)
}

pub trait HotkeyRegistry: Send + Sync {
    fn register(&self, id: &str, combo: &Combo) -> Result<(), RegistryError>;
    fn unregister(&self, id: &str) -> Result<(), RegistryError>;
    fn subscribe(&self) -> Receiver<ShortcutId>;
}

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("Неизвестная ошибка: {0}")]
    Internal(String),
}

pub trait Launcher: Send + Sync {
    fn launch(&self, action: &Action) -> Result<(), LaunchError>;
}
