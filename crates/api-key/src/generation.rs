use rand::{distributions::Alphanumeric, Rng};
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum KeyGenerationError {
    #[error("Invalid environment specified")]
    InvalidEnvironment,
    #[error("Failed to generate key")]
    GenerationFailed,
    #[error("Invalid key format")]
    InvalidFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Environment {
    Test,
    Live,
}

impl Environment {
    pub fn prefix(&self) -> &'static str {
        match self {
            Environment::Test => "tronch_sk_test_",
            Environment::Live => "tronch_sk_live_",
        }
    }
}

impl TryFrom<&str> for Environment {
    type Error = KeyGenerationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "test" => Ok(Environment::Test),
            "live" => Ok(Environment::Live),
            _ => Err(KeyGenerationError::InvalidEnvironment),
        }
    }
}

/// Generates a cryptographically secure API key with proper prefixing
/// 
/// # Arguments
/// * `env` - The environment (test/live) for the key
/// 
/// # Returns
/// * `Result<String, KeyGenerationError>` - The generated key or an error
/// 
/// # Example
/// ```
/// use tronch_api_key::generation::{generate_api_key, Environment};
/// 
/// let key = generate_api_key(Environment::Test).unwrap();
/// assert!(key.starts_with("tronch_sk_test_"));
/// ```
pub fn generate_api_key(env: Environment) -> Result<String, KeyGenerationError> {
    // Generate a timestamp component (8 chars)
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| KeyGenerationError::GenerationFailed)?
        .as_secs()
        .to_string()
        .chars()
        .rev()
        .take(8)
        .collect::<String>();

    // Calculate remaining length for random component
    let prefix_len = env.prefix().len();
    let random_len = 52 - prefix_len - 8; // Total length - prefix - timestamp

    // Generate a random component
    let random: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(random_len)
        .map(char::from)
        .collect();

    let key = format!("{}{}{}", env.prefix(), timestamp, random);
    
    // Validate the generated key
    validate_key_format(&key)?;
    
    Ok(key)
}

/// Validates the format of an API key
/// 
/// # Arguments
/// * `key` - The API key to validate
/// 
/// # Returns
/// * `Result<(), KeyGenerationError>` - Ok if valid, error if invalid
pub fn validate_key_format(key: &str) -> Result<(), KeyGenerationError> {
    // Check if key starts with valid prefix
    if !key.starts_with("tronch_sk_test_") && !key.starts_with("tronch_sk_live_") {
        return Err(KeyGenerationError::InvalidFormat);
    }

    // Check total length (should be 52 chars)
    if key.len() != 52 {
        return Err(KeyGenerationError::InvalidFormat);
    }

    Ok(())
}
