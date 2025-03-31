use tronch_api_key::{
    Environment,
    ApiKeyMetadata,
    validate_api_key,
};

mod common;
use common::create_test_key;

#[test]
fn test_valid_key() {
    let (key, metadata) = create_test_key();
    assert!(validate_api_key(&key, &metadata).is_ok());
}

#[test]
fn test_environment_mismatch() {
    let (key, _) = create_test_key();
    let wrong_env_metadata = ApiKeyMetadata::new(Environment::Live);
    assert!(validate_api_key(&key, &wrong_env_metadata).is_err());
}

#[test]
fn test_revoked_key() {
    let (key, mut metadata) = create_test_key();
    metadata.is_revoked = true;
    assert!(validate_api_key(&key, &metadata).is_err());
}

#[test]
fn test_inactive_key() {
    let (key, mut metadata) = create_test_key();
    metadata.is_active = false;
    assert!(validate_api_key(&key, &metadata).is_err());
}

#[test]
fn test_expired_key() {
    let (key, mut metadata) = create_test_key();
    metadata.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
    assert!(validate_api_key(&key, &metadata).is_err());
} 