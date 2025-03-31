use crate::rotation::*;
use crate::generation::Environment;
use crate::validation::ApiKeyMetadata;
use crate::storage::{InMemoryStorage, ApiKeyStorage};
use chrono::Duration;

async fn create_test_storage() -> InMemoryStorage {
    let storage = InMemoryStorage::new();
    let metadata = ApiKeyMetadata::new(Environment::Test, "test_key").unwrap();
    storage.store_key("test_key", metadata).await.unwrap();
    storage
}

#[tokio::test]
async fn test_key_rotation() {
    let storage = create_test_storage().await;
    let old_key = "test_key";
    
    let config = RotationConfig {
        grace_period: Duration::hours(24),
        auto_revoke: false,
    };

    let new_key = rotate_key(&storage, old_key, config).await.unwrap();
    assert!(new_key.starts_with("tronch_sk_test_"));

    // Old key should still work during grace period
    let old_metadata = storage.get_metadata(old_key).await.unwrap();
    assert!(old_metadata.is_active);
    assert!(!old_metadata.is_revoked);

    // New key should be active
    let new_metadata = storage.get_metadata(&new_key).await.unwrap();
    assert!(new_metadata.is_active);
    assert!(!new_metadata.is_revoked);
}

#[tokio::test]
async fn test_rotate_nonexistent_key() {
    let storage = create_test_storage().await;
    let config = RotationConfig {
        grace_period: Duration::hours(24),
        auto_revoke: false,
    };

    let result = rotate_key(&storage, "nonexistent", config).await;
    assert!(matches!(result, Err(KeyRotationError::KeyNotFound)));
}

#[tokio::test]
async fn test_rotate_revoked_key() {
    let storage = InMemoryStorage::new();
    let key = "test_key";
    let mut metadata = ApiKeyMetadata::new(Environment::Test, key).unwrap();
    metadata.is_revoked = true;
    storage.store_key(key, metadata).await.unwrap();

    let config = RotationConfig {
        grace_period: Duration::hours(24),
        auto_revoke: false,
    };

    let result = rotate_key(&storage, key, config).await;
    assert!(matches!(result, Err(KeyRotationError::KeyRevoked)));
} 