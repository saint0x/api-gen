use rand::{thread_rng, Rng, RngCore};
use sha2::{Sha256, Digest};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Test,
    Live,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Test => write!(f, "test"),
            Environment::Live => write!(f, "live"),
        }
    }
}

/// Generates a new API key with the specified environment prefix.
/// 
/// # Examples
/// ```
/// use tronch::generation::{generate_api_key, Environment};
/// 
/// let api_key = generate_api_key(Environment::Test);
/// assert!(api_key.starts_with("tronch_sk_test_"));
/// ```
pub fn generate_api_key(env: Environment) -> String {
    let mut rng = thread_rng();
    let mut random_bytes = [0u8; 32];
    rng.fill_bytes(&mut random_bytes);
    
    let mut hasher = Sha256::new();
    hasher.update(&random_bytes);
    let result = hasher.finalize();
    
    let key_suffix = hex::encode(&result[..16]); // Use first 16 bytes for reasonable key length
    format!("tronch_sk_{}_{}", env, key_suffix)
}

/// Validates that a key string matches the expected format.
pub fn validate_key_format(key: &str) -> bool {
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() != 4 {
        return false;
    }
    
    let [prefix, key_type, env, hash] = match parts.as_slice() {
        [a, b, c, d] => [a, b, c, d],
        _ => return false,
    };
    
    if prefix != "tronch" || key_type != "sk" {
        return false;
    }
    
    if env != "test" && env != "live" {
        return false;
    }
    
    // Validate hash part is 32 chars of hex
    if hash.len() != 32 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    
    true
} 