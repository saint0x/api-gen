use thiserror::Error;
use chrono::{DateTime, Utc};
use crate::generation::{Environment, validate_key_format};

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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyMetadata {
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub environment: Environment,
    pub is_active: bool,
    pub is_revoked: bool,
}

impl ApiKeyMetadata {
    pub fn new(environment: Environment) -> Self {
        Self {
            created_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
            environment,
            is_active: true,
            is_revoked: false,
        }
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
    // First validate the format
    validate_key_format(key).map_err(|_| ApiKeyValidationError::InvalidFormat)?;

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

    // Extract environment from key and check match
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::generate_api_key;

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
    fn test_revoked_key() {
        let key = generate_api_key(Environment::Test).unwrap();
        let mut metadata = ApiKeyMetadata::new(Environment::Test);
        metadata.is_revoked = true;
        
        assert!(matches!(
            validate_api_key(&key, &metadata),
            Err(ApiKeyValidationError::KeyRevoked)
        ));
    }

    #[test]
    fn test_inactive_key() {
        let key = generate_api_key(Environment::Test).unwrap();
        let mut metadata = ApiKeyMetadata::new(Environment::Test);
        metadata.is_active = false;
        
        assert!(matches!(
            validate_api_key(&key, &metadata),
            Err(ApiKeyValidationError::KeyInactive)
        ));
    }
}
