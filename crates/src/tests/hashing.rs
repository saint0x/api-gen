use crate::hashing::KeyHash;

#[test]
fn test_hash_creation() {
    let key = "tronch_sk_test_20240101abcdef1234567890abcdef1234567";
    let hash = KeyHash::new(key).unwrap();
    assert!(hash.verify(key).unwrap());
}

#[test]
fn test_hash_verification() {
    let key = "tronch_sk_test_20240101abcdef1234567890abcdef1234567";
    let hash = KeyHash::new(key).unwrap();
    
    // Correct key should verify
    assert!(hash.verify(key).unwrap());
    
    // Wrong key should not verify
    assert!(!hash.verify("wrong_key").unwrap());
    
    // Similar but different key should not verify
    let similar_key = "tronch_sk_test_20240101abcdef1234567890abcdef1234568";
    assert!(!hash.verify(similar_key).unwrap());
}

#[test]
fn test_hash_serialization() {
    let key = "tronch_sk_test_20240101abcdef1234567890abcdef1234567";
    let hash = KeyHash::new(key).unwrap();
    
    // Serialize and deserialize
    let serialized = hash.to_string();
    let deserialized = KeyHash::from_string(&serialized).unwrap();
    
    // Should still verify after serialization
    assert!(deserialized.verify(key).unwrap());
}

#[test]
fn test_invalid_hash_format() {
    // Invalid format should error
    assert!(KeyHash::from_string("invalid").is_err());
    
    // Too many parts should error
    assert!(KeyHash::from_string("too:many:parts").is_err());
    
    // Empty string should error
    assert!(KeyHash::from_string("").is_err());
}

#[test]
fn test_hash_uniqueness() {
    let key = "tronch_sk_test_20240101abcdef1234567890abcdef1234567";
    
    // Create two hashes of the same key
    let hash1 = KeyHash::new(key).unwrap();
    let hash2 = KeyHash::new(key).unwrap();
    
    // Hashes should be different due to different salts
    assert_ne!(hash1.to_string(), hash2.to_string());
    
    // Both should verify the original key
    assert!(hash1.verify(key).unwrap());
    assert!(hash2.verify(key).unwrap());
}

#[test]
fn test_hash_with_special_chars() {
    let key = "tronch_sk_test_20240101!@#$%^&*()_+abcdef123456789";
    let hash = KeyHash::new(key).unwrap();
    assert!(hash.verify(key).unwrap());
}

#[test]
fn test_hash_with_unicode() {
    let key = "tronch_sk_test_20240101🌍🌎🌏abcdef123456789";
    let hash = KeyHash::new(key).unwrap();
    assert!(hash.verify(key).unwrap());
} 