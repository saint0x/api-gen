use std::collections::HashMap;
use tokio::sync::Mutex;
use thiserror::Error;
use crate::validation::ApiKeyMetadata;
use crate::generation::Environment;
use crate::hashing::HashingError;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Key already exists")]
    KeyExists,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Hash error: {0}")]
    HashError(#[from] HashingError),
}

/// Trait defining the storage interface for API keys
#[async_trait::async_trait]
pub trait ApiKeyStorage: Send + Sync + std::fmt::Debug {
    /// Store a new API key with its metadata
    async fn store_key(&self, key: &str, metadata: ApiKeyMetadata) -> Result<(), StorageError>;
    
    /// Retrieve metadata for an API key
    async fn get_metadata(&self, key: &str) -> Result<ApiKeyMetadata, StorageError>;
    
    /// Update metadata for an existing API key
    async fn update_metadata(&self, key: &str, metadata: ApiKeyMetadata) -> Result<(), StorageError>;
    
    /// Delete an API key
    async fn delete_key(&self, key: &str) -> Result<(), StorageError>;
    
    /// List all API keys for an environment
    async fn list_keys(&self, environment: Environment) -> Result<Vec<String>, StorageError>;
}

/// In-memory storage implementation for testing
#[derive(Default, Debug)]
pub struct InMemoryStorage {
    keys: Mutex<HashMap<String, ApiKeyMetadata>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            keys: Mutex::new(HashMap::new()),
        }
    }

    async fn find_by_hash(&self, key: &str) -> Result<Option<(String, ApiKeyMetadata)>, StorageError> {
        let keys = self.keys.lock().await;
        let mut result = None;
        
        for (stored_key, metadata) in keys.iter() {
            if metadata.verify_key(key).map_err(StorageError::HashError)? {
                result = Some((stored_key.clone(), metadata.clone()));
                break;
            }
        }
        
        Ok(result)
    }
}

#[async_trait::async_trait]
impl ApiKeyStorage for InMemoryStorage {
    async fn store_key(&self, key: &str, metadata: ApiKeyMetadata) -> Result<(), StorageError> {
        // Check if key exists first
        if let Some(_) = self.find_by_hash(key).await? {
            return Err(StorageError::KeyExists);
        }
        
        // Store the key
        let mut keys = self.keys.lock().await;
        keys.insert(key.to_string(), metadata);
        Ok(())
    }

    async fn get_metadata(&self, key: &str) -> Result<ApiKeyMetadata, StorageError> {
        match self.find_by_hash(key).await? {
            Some((_, metadata)) => Ok(metadata),
            None => Err(StorageError::KeyNotFound),
        }
    }

    async fn update_metadata(&self, key: &str, metadata: ApiKeyMetadata) -> Result<(), StorageError> {
        // Find the key first
        let stored_key = match self.find_by_hash(key).await? {
            Some((k, _)) => k,
            None => return Err(StorageError::KeyNotFound),
        };
        
        // Update the metadata
        let mut keys = self.keys.lock().await;
        keys.insert(stored_key, metadata);
        Ok(())
    }

    async fn delete_key(&self, key: &str) -> Result<(), StorageError> {
        // Find the key first
        let stored_key = match self.find_by_hash(key).await? {
            Some((k, _)) => k,
            None => return Err(StorageError::KeyNotFound),
        };
        
        // Delete the key
        let mut keys = self.keys.lock().await;
        keys.remove(&stored_key);
        Ok(())
    }

    async fn list_keys(&self, environment: Environment) -> Result<Vec<String>, StorageError> {
        let keys = self.keys.lock().await;
        Ok(keys
            .iter()
            .filter(|(_, metadata)| metadata.environment == environment)
            .map(|(key, _)| key.clone())
            .collect())
    }
}
