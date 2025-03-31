use crate::storage::*;
use crate::validation::ApiKeyMetadata;
use crate::generation::Environment;
use chrono::Utc;

#[tokio::test]
async fn test_store_and_get_key() {
    let storage = InMemoryStorage::new();
    let key = "test_key";
    let metadata = ApiKeyMetadata {
        created_at: Utc::now(),
        last_used_at: None,
        expires_at: None,
        environment: Environment::Test,
        is_active: true,
        is_revoked: false,
    };

    storage.store_key(key, metadata.clone()).await.unwrap();
    let retrieved = storage.get_metadata(key).await.unwrap();
    assert_eq!(retrieved.environment, metadata.environment);
    assert_eq!(retrieved.is_active, metadata.is_active);
}

#[tokio::test]
async fn test_get_nonexistent_key() {
    let storage = InMemoryStorage::new();
    let result = storage.get_metadata("nonexistent").await;
    assert!(matches!(result, Err(StorageError::KeyNotFound)));
}

#[tokio::test]
async fn test_store_duplicate_key() {
    let storage = InMemoryStorage::new();
    let key = "test_key";
    let metadata = ApiKeyMetadata {
        created_at: Utc::now(),
        last_used_at: None,
        expires_at: None,
        environment: Environment::Test,
        is_active: true,
        is_revoked: false,
    };

    storage.store_key(key, metadata.clone()).await.unwrap();
    let result = storage.store_key(key, metadata).await;
    assert!(matches!(result, Err(StorageError::KeyExists)));
}

#[tokio::test]
async fn test_update_metadata() {
    let storage = InMemoryStorage::new();
    let key = "test_key";
    let mut metadata = ApiKeyMetadata {
        created_at: Utc::now(),
        last_used_at: None,
        expires_at: None,
        environment: Environment::Test,
        is_active: true,
        is_revoked: false,
    };

    storage.store_key(key, metadata.clone()).await.unwrap();
    
    metadata.is_active = false;
    storage.update_metadata(key, metadata.clone()).await.unwrap();
    
    let updated = storage.get_metadata(key).await.unwrap();
    assert_eq!(updated.is_active, false);
}

#[tokio::test]
async fn test_list_keys() {
    let storage = InMemoryStorage::new();
    let test_key = "test_key";
    let live_key = "live_key";
    
    let test_metadata = ApiKeyMetadata {
        created_at: Utc::now(),
        last_used_at: None,
        expires_at: None,
        environment: Environment::Test,
        is_active: true,
        is_revoked: false,
    };
    
    let live_metadata = ApiKeyMetadata {
        created_at: Utc::now(),
        last_used_at: None,
        expires_at: None,
        environment: Environment::Live,
        is_active: true,
        is_revoked: false,
    };

    storage.store_key(test_key, test_metadata).await.unwrap();
    storage.store_key(live_key, live_metadata).await.unwrap();

    let test_keys = storage.list_keys(Environment::Test).await.unwrap();
    assert_eq!(test_keys.len(), 1);
    assert_eq!(test_keys[0], test_key);

    let live_keys = storage.list_keys(Environment::Live).await.unwrap();
    assert_eq!(live_keys.len(), 1);
    assert_eq!(live_keys[0], live_key);
}