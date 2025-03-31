use thiserror::Error;
use chrono::{Duration, Utc};
use crate::{
    generation::generate_api_key,
    storage::ApiKeyStorage,
};

#[derive(Error, Debug)]
pub enum KeyRotationError {
    #[error("Failed to generate new key")]
    GenerationFailed,
    #[error("Failed to store new key")]
    StorageFailed,
    #[error("Failed to revoke old key")]
    RevocationFailed,
    #[error("Invalid grace period")]
    InvalidGracePeriod,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Key is revoked")]
    KeyRevoked,
}

/// Configuration for key rotation
#[derive(Debug, Clone)]
pub struct RotationConfig {
    /// How long to keep the old key valid after rotation
    pub grace_period: Duration,
    /// Whether to automatically revoke old keys after grace period
    pub auto_revoke: bool,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            grace_period: Duration::days(7),
            auto_revoke: true,
        }
    }
}

/// Rotates an API key, creating a new one and optionally invalidating the old one
/// 
/// # Arguments
/// * `storage` - The storage backend for key management
/// * `old_key` - The key to rotate
/// * `config` - Rotation configuration
/// 
/// # Returns
/// * `Result<String, KeyRotationError>` - The new key or an error
pub async fn rotate_key(
    storage: &impl ApiKeyStorage,
    old_key: &str,
    config: RotationConfig,
) -> Result<String, KeyRotationError> {
    // Get metadata for old key
    let metadata = storage
        .get_metadata(old_key)
        .await
        .map_err(|_| KeyRotationError::KeyNotFound)?;

    // Check if key is revoked
    if metadata.is_revoked {
        return Err(KeyRotationError::KeyRevoked);
    }

    // Generate new key in same environment
    let (new_key, new_metadata) = generate_api_key(metadata.environment)
        .map_err(|_| KeyRotationError::GenerationFailed)?;

    // Store new key
    storage
        .store_key(&new_key, new_metadata)
        .await
        .map_err(|_| KeyRotationError::StorageFailed)?;

    // Update old key metadata with grace period
    let mut old_metadata = metadata;
    old_metadata.expires_at = Some(Utc::now() + config.grace_period);
    if config.auto_revoke {
        old_metadata.is_revoked = true;
    }

    // Update old key metadata
    storage
        .update_metadata(old_key, old_metadata)
        .await
        .map_err(|_| KeyRotationError::RevocationFailed)?;

    Ok(new_key)
} 