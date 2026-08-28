use crate::domain::hotkey::Shortcut;
use crate::domain::repository::{RepoError, ShortcutRepository};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use uuid::Uuid;

/// JSON file-backed repository (MVP storage).
#[derive(Debug)]
pub struct JsonShortcutRepository {
    path: PathBuf,
    by_id: RwLock<HashMap<Uuid, Shortcut>>,
}

impl JsonShortcutRepository {
    pub fn load_or_default(path: PathBuf) -> Result<Self, RepoError> {
        let map = load_map(&path)?;
        Ok(Self {
            path,
            by_id: RwLock::new(map),
        })
    }
}

impl ShortcutRepository for JsonShortcutRepository {
    fn get(&self, id: &Uuid) -> Result<Option<Shortcut>, RepoError> {
        let guard = self
            .by_id
            .read()
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        Ok(guard.get(id).cloned())
    }

    fn all(&self) -> Result<Vec<Shortcut>, RepoError> {
        let guard = self
            .by_id
            .read()
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        Ok(guard.values().cloned().collect())
    }

    fn save(&self, shortcut: &Shortcut) -> Result<(), RepoError> {
        let mut guard = self
            .by_id
            .write()
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        guard.insert(shortcut.id.clone(), shortcut.clone());
        persist_map(&self.path, &guard)
    }

    fn delete(&self, id: &Uuid) -> Result<(), RepoError> {
        let mut guard = self
            .by_id
            .write()
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        guard.remove(id);
        persist_map(&self.path, &guard)
    }
}

fn load_map(path: &Path) -> Result<HashMap<Uuid, Shortcut>, RepoError> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(e) => return Err(RepoError::Internal(e.to_string())),
    };

    let list: Vec<Shortcut> =
        serde_json::from_slice(&bytes).map_err(|e| RepoError::Internal(e.to_string()))?;

    Ok(list.into_iter().map(|s| (s.id.clone(), s)).collect())
}

fn persist_map(path: &Path, map: &HashMap<Uuid, Shortcut>) -> Result<(), RepoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| RepoError::Internal(e.to_string()))?;
    }

    let mut list: Vec<Shortcut> = map.values().cloned().collect();
    list.sort_by(|a, b| a.id.cmp(&b.id));

    let data = serde_json::to_vec_pretty(&list).map_err(|e| RepoError::Internal(e.to_string()))?;

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, data).map_err(|e| RepoError::Internal(e.to_string()))?;

    if path.exists() {
        fs::remove_file(path).map_err(|e| RepoError::Internal(e.to_string()))?;
    }

    fs::rename(&tmp_path, path).map_err(|e| RepoError::Internal(e.to_string()))?;
    Ok(())
}
