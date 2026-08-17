use crate::domain::hotkey::Shortcut;
use crate::domain::repository::{RepoError, ShortcutRepository};

pub struct ConfigStorage{
    
}

impl ShortcutRepository for ConfigStorage {
    fn get(&self, id: &str) -> Result<Option<Shortcut>, RepoError> {
        todo!()
    }

    fn all(&self) -> Result<Vec<Shortcut>, RepoError> {
        todo!()
    }

    fn save(&self, shortcut: &Shortcut) -> Result<(), RepoError> {
        todo!()
    }

    fn delete(&self, id: &str) -> Result<(), RepoError> {
        todo!()
    }
}