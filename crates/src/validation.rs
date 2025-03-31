use thiserror::Error;
use chrono::{DateTime, Utc};
use crate::generation::{Environment, validate_key_format, KeyGenerationError};
use crate::hashing::{KeyHash, HashingError};

#[derive(Error, Debug)]
pub enum ApiKeyValidationError {
    #[error("Invalid key format")]
    InvalidFormat,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Key is expired")]
    KeyExpired,
    #[error("Key is revoked")]
    KeyRevoked,
    #[error("Key is inactive")]
    KeyInactive,
    #[error("Environment mismatch")]
    EnvironmentMismatch,
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Hash verification failed")]
    HashVerificationFailed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyMetadata {
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub environment: Environment,
    pub is_active: bool,
    pub is_revoked: bool,
    pub key_hash: String, // Store serialized hash
}

impl ApiKeyMetadata {
    pub fn new(environment: Environment, key: &str) -> Result<Self, HashingError> {
        let key_hash = KeyHash::new(key)?;
        Ok(Self {
            created_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
            environment,
            is_active: true,
            is_revoked: false,
            key_hash: key_hash.to_string(),
        })
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_revoked && !self.is_expired()
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    pub fn verify_key(&self, key: &str) -> Result<bool, HashingError> {
        let key_hash = KeyHash::from_string(&self.key_hash)?;
        key_hash.verify(key)
    }
}

/// Validates an API key's existence and status
/// 
/// # Arguments
/// * `key` - The API key to validate
/// * `metadata` - The key's metadata
/// 
/// # Returns
/// * `Result<(), ApiKeyValidationError>` - Ok if valid, error if invalid
pub fn validate_api_key(key: &str, metadata: &ApiKeyMetadata) -> Result<(), ApiKeyValidationError> {
    // First validate the format and environment
    match validate_key_format(key, Some(metadata.environment)) {
        Ok(_) => {}
        Err(KeyGenerationError::InvalidFormat) => {
            // Check if it's an environment mismatch
            let key_env = if key.starts_with("tronch_sk_test_") {
                Environment::Test
            } else if key.starts_with("tronch_sk_live_") {
                Environment::Live
            } else {
                return Err(ApiKeyValidationError::InvalidFormat);
            };

            if key_env != metadata.environment {
                return Err(ApiKeyValidationError::EnvironmentMismatch);
            }
            return Err(ApiKeyValidationError::InvalidFormat);
        }
        Err(_) => return Err(ApiKeyValidationError::InvalidFormat),
    }

    // Verify the key hash
    match metadata.verify_key(key) {
        Ok(true) => {},
        Ok(false) => return Err(ApiKeyValidationError::InvalidFormat),
        Err(_) => return Err(ApiKeyValidationError::HashVerificationFailed),
    }

    // Check key status
    if !metadata.is_active {
        return Err(ApiKeyValidationError::KeyInactive);
    }

    if metadata.is_revoked {
        return Err(ApiKeyValidationError::KeyRevoked);
    }

    if metadata.is_expired() {
        return Err(ApiKeyValidationError::KeyExpired);
    }

    Ok(())
}
