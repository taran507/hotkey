use crate::domain::repository::{ResolverError, SystemResolver};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct Resolver(Mutex<HashMap<u32, Uuid>>);

impl Resolver {
    pub fn new() -> Arc<Self> {
        Arc::new(Resolver(Mutex::new(HashMap::new())))
    }
}

impl SystemResolver for Resolver {
    fn add(&self, system_id: u32, id: Uuid) -> Result<(), ResolverError> {
        self.0
            .lock()
            .map_err(|e| ResolverError::Internal(e.to_string()))?
            .insert(system_id, id);
        Ok(())
    }

    fn remove(&self, system_id: &u32) -> Result<(), ResolverError> {
        self.0
            .lock()
            .map_err(|e| ResolverError::Internal(e.to_string()))?
            .remove(system_id);
        Ok(())
    }

    fn resolve(&self, system_id: &u32) -> Result<Option<Uuid>, ResolverError> {
        let id = self
            .0
            .lock()
            .map_err(|e| ResolverError::Internal(e.to_string()))?
            .get(system_id)
            .cloned();
        Ok(id)
    }
}
