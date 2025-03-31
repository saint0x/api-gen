use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashingError {
    #[error("Failed to hash key: {0}")]
    HashError(String),
    #[error("Failed to verify key: {0}")]
    VerifyError(String),
}

/// Represents a hashed API key with its salt
#[derive(Debug, Clone)]
pub struct KeyHash {
    hash: String,
    salt: String,
}

impl KeyHash {
    /// Creates a new KeyHash from an API key
    pub fn new(key: &str) -> Result<Self, HashingError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let hash = argon2
            .hash_password(key.as_bytes(), &salt)
            .map_err(|e| HashingError::HashError(e.to_string()))?;

        Ok(Self {
            hash: hash.to_string(),
            salt: salt.to_string(),
        })
    }

    /// Verifies a key against this hash
    pub fn verify(&self, key: &str) -> Result<bool, HashingError> {
        let hash = PasswordHash::new(&self.hash)
            .map_err(|e| HashingError::VerifyError(e.to_string()))?;

        match Argon2::default().verify_password(key.as_bytes(), &hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(HashingError::VerifyError(e.to_string())),
        }
    }

    /// Serializes the hash for storage
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.salt, self.hash)
    }

    /// Deserializes a hash from storage
    pub fn from_string(s: &str) -> Result<Self, HashingError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(HashingError::VerifyError("Invalid hash format".to_string()));
        }

        Ok(Self {
            salt: parts[0].to_string(),
            hash: parts[1].to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_hash_creation_and_verification() {
        let key = "tronch_sk_test_20240101abcdef1234567890abcdef1234567";
        let hash = KeyHash::new(key).unwrap();
        
        assert!(hash.verify(key).unwrap());
        assert!(!hash.verify("wrong_key").unwrap());
    }

    #[test]
    fn test_hash_serialization() {
        let key = "tronch_sk_test_20240101abcdef1234567890abcdef1234567";
        let hash = KeyHash::new(key).unwrap();
        
        let serialized = hash.to_string();
        let deserialized = KeyHash::from_string(&serialized).unwrap();
        
        assert!(deserialized.verify(key).unwrap());
    }

    #[test]
    fn test_invalid_hash_format() {
        assert!(KeyHash::from_string("invalid").is_err());
        assert!(KeyHash::from_string("too:many:parts").is_err());
    }
} 