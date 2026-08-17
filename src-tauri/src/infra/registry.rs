use std::sync::mpsc::Receiver;
use crate::domain::hotkey::{Combo, ShortcutId};
use crate::domain::repository::{HotkeyRegistry, RegistryError};

pub struct Registry{

}

impl HotkeyRegistry for Registry {
    fn register(&self, id: &str, combo: &Combo) -> Result<(), RegistryError> {
        todo!()
    }

    fn unregister(&self, id: &str) -> Result<(), RegistryError> {
        todo!()
    }

    fn subscribe(&self) -> Receiver<ShortcutId> {
        todo!()
    }
}