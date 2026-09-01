use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Пустое название")]
    EmptyName,
    #[error("Пустой ключ")]
    EmptyKey,
    #[error("Нет клавиш модификаторов")]
    NoModifier,
    #[error("Передана невалидная программа для запуска")]
    NoProgram,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shortcut {
    pub id: Uuid,
    pub name: String,
    pub combo: Combo,
    pub action: Action,
    pub enabled: bool,
    pub created_at: i64,
}

impl Shortcut {
    pub fn new(name: String, combo: Combo, action: Action) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::EmptyName);
        }

        combo.validate()?;
        action.validate()?;
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            combo,
            action,
            enabled: true,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    pub fn update(
        &mut self,
        name: String,
        combo: Combo,
        action: Action,
        enable: bool,
    ) -> Result<(), DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::EmptyName);
        }
        combo.validate()?;
        action.validate()?;
        self.name = name;
        self.combo = combo;
        self.action = action;
        self.enabled = enable;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Combo {
    pub key: PhysicalKey,
    pub mods: Mods,
}

impl Combo {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.key.0.trim().is_empty() {
            return Err(DomainError::EmptyKey);
        }

        // For non-function keys (e.g. letters), require at least one modifier to avoid conflicts.
        if self.mods.is_empty() && !self.key.is_func_key() {
            return Err(DomainError::NoModifier);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalKey(pub String);

impl PhysicalKey {
    fn is_func_key(&self) -> bool {
        self.0.starts_with("F") && self.0.len() > 1
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub logo: bool,
}

impl Mods {
    fn is_empty(&self) -> bool {
        !(self.ctrl || self.alt || self.shift || self.logo)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Launch { program: PathBuf, args: Vec<String> },
}

impl Action {
    fn validate(&self) -> Result<(), DomainError> {
        match self {
            Action::Launch { program, .. } => {
                if program.as_os_str().is_empty() || !can_open_program(program) {
                    return Err(DomainError::NoProgram);
                }
            }
        }
        Ok(())
    }
}

fn can_open_program(program: &Path) -> bool {
    fs::metadata(program).is_ok_and(|meta| meta.is_file()) || which::which(program).is_ok()
}
