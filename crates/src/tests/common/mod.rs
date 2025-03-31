use crate::storage::{ApiKeyStorage, InMemoryStorage};
use std::sync::Arc;

/// Creates a test API key with metadata
#[allow(dead_code)]
pub fn create_test_key() -> String {
    "tronch_sk_test_1234567890abcdef".to_string()
}

/// Creates a test storage with a single key
#[allow(dead_code)]
pub fn create_test_storage() -> Arc<dyn ApiKeyStorage> {
    Arc::new(InMemoryStorage::new())
} 