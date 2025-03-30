use tronch_api_key::{
    Environment,
    ApiKeyMetadata,
    InMemoryStorage,
    ApiKeyStorage,
    generate_api_key,
};

/// Creates a test API key with metadata
#[allow(dead_code)]
pub fn create_test_key() -> (String, ApiKeyMetadata) {
    let key = generate_api_key(Environment::Test).unwrap();
    let metadata = ApiKeyMetadata::new(Environment::Test);
    (key, metadata)
}

/// Creates a test storage with a single key
#[allow(dead_code)]
pub fn create_test_storage() -> (InMemoryStorage, String, ApiKeyMetadata) {
    let storage = InMemoryStorage::new();
    let (key, metadata) = create_test_key();
    storage.store_key(&key, metadata.clone()).unwrap();
    (storage, key, metadata)
} 