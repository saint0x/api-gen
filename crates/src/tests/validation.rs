use crate::validation::*;
use crate::generation::{generate_api_key, Environment};

fn create_test_key() -> (String, ApiKeyMetadata) {
    let prefix = "tronch_sk_test_";  // 15 chars
    let timestamp = "20240101";      // 8 chars
    // 52 - 15 - 8 = 29 chars needed for random
    let random = "abcdef1234567890abcdef1234567";  // exactly 29 chars
    let key = format!("{}{}{}", prefix, timestamp, random);
    let metadata = ApiKeyMetadata::new(Environment::Test);
    (key, metadata)
}

#[test]
fn test_valid_key() {
    let (key, metadata) = create_test_key();
    assert_eq!(key.len(), 52); // Verify length
    assert!(key.starts_with("tronch_sk_test_")); // Verify prefix
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

#[test]
fn test_key_validation() {
    let key = generate_api_key(Environment::Test).unwrap();
    let metadata = ApiKeyMetadata::new(Environment::Test);
    assert!(validate_api_key(&key, &metadata).is_ok());
}

#[test]
fn test_invalid_environment() {
    let key = generate_api_key(Environment::Test).unwrap();
    let metadata = ApiKeyMetadata::new(Environment::Live);
    assert!(matches!(
        validate_api_key(&key, &metadata),
        Err(ApiKeyValidationError::EnvironmentMismatch)
    ));
}

#[test]
fn test_revoked_key_generated() {
    let key = generate_api_key(Environment::Test).unwrap();
    let mut metadata = ApiKeyMetadata::new(Environment::Test);
    metadata.is_revoked = true;
    assert!(matches!(
        validate_api_key(&key, &metadata),
        Err(ApiKeyValidationError::KeyRevoked)
    ));
}

#[test]
fn test_inactive_key_generated() {
    let key = generate_api_key(Environment::Test).unwrap();
    let mut metadata = ApiKeyMetadata::new(Environment::Test);
    metadata.is_active = false;
    assert!(matches!(
        validate_api_key(&key, &metadata),
        Err(ApiKeyValidationError::KeyInactive)
    ));
} 