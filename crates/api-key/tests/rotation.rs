use tronch_api_key::{
    Environment,
    ApiKeyMetadata,
    InMemoryStorage,
    ApiKeyStorage,
    generate_api_key,
    rotate_key,
    RotationConfig,
};
use chrono::Duration;

#[tokio::test]
async fn test_key_rotation() {
    let storage = InMemoryStorage::new();
    let old_key = generate_api_key(Environment::Test).unwrap();
    let metadata = ApiKeyMetadata::new(Environment::Test);
    storage.store_key(&old_key, metadata).await.unwrap();

    let config = RotationConfig::default();
    let new_key = rotate_key(&storage, &old_key, config).await.unwrap();

    // Verify new key exists and is valid
    assert!(storage.get_metadata(&new_key).await.is_ok());
    
    // Verify old key has grace period
    let old_metadata = storage.get_metadata(&old_key).await.unwrap();
    assert!(old_metadata.expires_at.is_some());
    assert!(old_metadata.is_revoked);
}

#[tokio::test]
async fn test_key_rotation_without_revoke() {
    let storage = InMemoryStorage::new();
    let old_key = generate_api_key(Environment::Test).unwrap();
    let metadata = ApiKeyMetadata::new(Environment::Test);
    storage.store_key(&old_key, metadata).await.unwrap();

    let config = RotationConfig {
        grace_period: Duration::days(7),
        auto_revoke: false,
    };
    let new_key = rotate_key(&storage, &old_key, config).await.unwrap();

    // Verify new key exists and is valid
    assert!(storage.get_metadata(&new_key).await.is_ok());
    
    // Verify old key has grace period but is not revoked
    let old_metadata = storage.get_metadata(&old_key).await.unwrap();
    assert!(old_metadata.expires_at.is_some());
    assert!(!old_metadata.is_revoked);
}

#[tokio::test]
async fn test_key_rotation_environment_preservation() {
    let storage = InMemoryStorage::new();
    let old_key = generate_api_key(Environment::Live).unwrap();
    let metadata = ApiKeyMetadata::new(Environment::Live);
    storage.store_key(&old_key, metadata).await.unwrap();

    let config = RotationConfig::default();
    let new_key = rotate_key(&storage, &old_key, config).await.unwrap();

    // Verify new key is in the same environment
    assert!(new_key.starts_with("tronch_sk_live_"));
}

#[tokio::test]
async fn test_key_rotation_nonexistent_key() {
    let storage = InMemoryStorage::new();
    let config = RotationConfig::default();
    
    // Attempt to rotate a non-existent key
    let result = rotate_key(&storage, "nonexistent_key", config).await;
    assert!(result.is_err());
} 