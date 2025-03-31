use tronch_api_key::{
    Environment,
    ApiKeyStorage,
    StorageError,
    ApiKeyMetadata,
    InMemoryStorage,
};

mod common;
use common::create_test_storage;

#[tokio::test]
async fn test_store_and_retrieve() {
    let storage = InMemoryStorage::new();
    let key = "test_key".to_string();
    let metadata = ApiKeyMetadata::new(Environment::Test);

    // Store key
    storage.store_key(&key, metadata.clone()).await.unwrap();

    // Retrieve metadata
    let retrieved = storage.get_metadata(&key).await.unwrap();
    assert_eq!(retrieved.environment, metadata.environment);
    assert_eq!(retrieved.is_active, metadata.is_active);
    assert_eq!(retrieved.is_revoked, metadata.is_revoked);
}

#[tokio::test]
async fn test_duplicate_key() {
    let (storage, key, metadata) = create_test_storage().await;
    
    // Attempt to store the same key again
    assert!(matches!(
        storage.store_key(&key, metadata).await,
        Err(StorageError::KeyExists)
    ));
}

#[tokio::test]
async fn test_update_metadata() {
    let storage = InMemoryStorage::new();
    let key = "test_key".to_string();
    let mut metadata = ApiKeyMetadata::new(Environment::Test);

    // Store initial metadata
    storage.store_key(&key, metadata.clone()).await.unwrap();

    // Update metadata
    metadata.is_active = false;
    storage.update_metadata(&key, metadata.clone()).await.unwrap();

    // Verify update
    let retrieved = storage.get_metadata(&key).await.unwrap();
    assert!(!retrieved.is_active);
}

#[tokio::test]
async fn test_delete_key() {
    let storage = InMemoryStorage::new();
    let key = "test_key".to_string();
    let metadata = ApiKeyMetadata::new(Environment::Test);

    // Store key
    storage.store_key(&key, metadata).await.unwrap();

    // Delete key
    storage.delete_key(&key).await.unwrap();

    // Verify deletion
    assert!(storage.get_metadata(&key).await.is_err());
}

#[tokio::test]
async fn test_list_keys() {
    let storage = InMemoryStorage::new();
    let key1 = "test_key1".to_string();
    let key2 = "test_key2".to_string();
    let metadata = ApiKeyMetadata::new(Environment::Test);

    // Store multiple keys
    storage.store_key(&key1, metadata.clone()).await.unwrap();
    storage.store_key(&key2, metadata).await.unwrap();

    // List keys
    let keys = storage.list_keys(Environment::Test).await.unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&key1));
    assert!(keys.contains(&key2));
} 