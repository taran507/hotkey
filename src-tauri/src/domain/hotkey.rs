use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("пустое название")]
    EmptyName,
    #[error("пустой ключ")]
    EmptyKey,
    #[error("No Modifier")]
    NoModifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shortcut {
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    pub combo: Combo,
    pub action: Action,
    pub enabled: bool,
}

impl Shortcut {
    pub fn new(name: String, combo: Combo, action: Action) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::EmptyName);
        }

        combo.validate()?;
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            combo,
            action,
            enabled: true,
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
    pub fn id(&self) -> String {
        format!(
            "{}|ctrl:{}|alt:{}|shift:{}|logo:{}",
            self.key.0, self.mods.ctrl, self.mods.alt, self.mods.shift, self.mods.logo
        )
    }

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
