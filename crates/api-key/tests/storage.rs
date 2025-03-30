use tronch_api_key::{
    Environment,
    ApiKeyStorage,
    StorageError,
};

mod common;
use common::{create_test_key, create_test_storage};

#[test]
fn test_store_and_retrieve() {
    let (storage, key, metadata) = create_test_storage();
    
    let retrieved = storage.get_metadata(&key).unwrap();
    assert_eq!(retrieved.environment, metadata.environment);
}

#[test]
fn test_duplicate_key() {
    let (storage, key, metadata) = create_test_storage();
    
    // Attempt to store the same key again
    assert!(matches!(
        storage.store_key(&key, metadata),
        Err(StorageError::KeyExists)
    ));
}

#[test]
fn test_update_metadata() {
    let (storage, key, mut metadata) = create_test_storage();
    
    // Update metadata
    metadata.is_active = false;
    assert!(storage.update_metadata(&key, metadata.clone()).is_ok());
    
    // Verify update
    let retrieved = storage.get_metadata(&key).unwrap();
    assert_eq!(retrieved.is_active, false);
}

#[test]
fn test_delete_key() {
    let (storage, key, _) = create_test_storage();
    
    // Delete key
    assert!(storage.delete_key(&key).is_ok());
    
    // Verify deletion
    assert!(matches!(
        storage.get_metadata(&key),
        Err(StorageError::KeyNotFound)
    ));
}

#[test]
fn test_list_keys() {
    let storage = create_test_storage().0;
    
    // Create additional test key
    let (key2, metadata2) = create_test_key();
    storage.store_key(&key2, metadata2).unwrap();
    
    // List test environment keys
    let keys = storage.list_keys(Environment::Test).unwrap();
    assert_eq!(keys.len(), 2);
    
    // List live environment keys (should be empty)
    let live_keys = storage.list_keys(Environment::Live).unwrap();
    assert!(live_keys.is_empty());
} 