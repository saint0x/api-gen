use sha2::{Sha256, Digest};

// Function to hash the API key using SHA-256
pub fn hash_api_key(api_key: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key);
    let hash = hasher.finalize();
    format!("{:x}", hash)
}
