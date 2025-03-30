use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use crate::validation::ApiKeyMetadata;
use crate::generation::Environment;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Key already exists")]
    KeyExists,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Trait defining the storage interface for API keys
pub trait ApiKeyStorage: Send + Sync {
    /// Store a new API key with its metadata
    fn store_key(&self, key: &str, metadata: ApiKeyMetadata) -> Result<(), StorageError>;
    
    /// Retrieve metadata for an API key
    fn get_metadata(&self, key: &str) -> Result<ApiKeyMetadata, StorageError>;
    
    /// Update metadata for an existing API key
    fn update_metadata(&self, key: &str, metadata: ApiKeyMetadata) -> Result<(), StorageError>;
    
    /// Delete an API key
    fn delete_key(&self, key: &str) -> Result<(), StorageError>;
    
    /// List all API keys for an environment
    fn list_keys(&self, environment: Environment) -> Result<Vec<String>, StorageError>;
}

/// In-memory storage implementation for testing
#[derive(Default)]
pub struct InMemoryStorage {
    keys: Mutex<HashMap<String, ApiKeyMetadata>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            keys: Mutex::new(HashMap::new()),
        }
    }
}

impl ApiKeyStorage for InMemoryStorage {
    fn store_key(&self, key: &str, metadata: ApiKeyMetadata) -> Result<(), StorageError> {
        let mut keys = self.keys.lock().map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        if keys.contains_key(key) {
            return Err(StorageError::KeyExists);
        }
        
        keys.insert(key.to_string(), metadata);
        Ok(())
    }

    fn get_metadata(&self, key: &str) -> Result<ApiKeyMetadata, StorageError> {
        let keys = self.keys.lock().map_err(|e| StorageError::StorageError(e.to_string()))?;
        keys.get(key)
            .cloned()
            .ok_or(StorageError::KeyNotFound)
    }

    fn update_metadata(&self, key: &str, metadata: ApiKeyMetadata) -> Result<(), StorageError> {
        let mut keys = self.keys.lock().map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        if !keys.contains_key(key) {
            return Err(StorageError::KeyNotFound);
        }
        
        keys.insert(key.to_string(), metadata);
        Ok(())
    }

    fn delete_key(&self, key: &str) -> Result<(), StorageError> {
        let mut keys = self.keys.lock().map_err(|e| StorageError::StorageError(e.to_string()))?;
        
        if !keys.contains_key(key) {
            return Err(StorageError::KeyNotFound);
        }
        
        keys.remove(key);
        Ok(())
    }

    fn list_keys(&self, environment: Environment) -> Result<Vec<String>, StorageError> {
        let keys = self.keys.lock().map_err(|e| StorageError::StorageError(e.to_string()))?;
        Ok(keys
            .iter()
            .filter(|(_, metadata)| metadata.environment == environment)
            .map(|(key, _)| key.clone())
            .collect())
    }
}
