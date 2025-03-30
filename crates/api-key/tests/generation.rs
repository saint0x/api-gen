use tronch_api_key::{
    Environment,
    generate_api_key,
    validate_key_format,
};

mod common;

#[test]
fn test_key_generation() {
    let test_key = generate_api_key(Environment::Test).unwrap();
    let live_key = generate_api_key(Environment::Live).unwrap();

    assert!(test_key.starts_with("tronch_sk_test_"));
    assert!(live_key.starts_with("tronch_sk_live_"));
    assert_eq!(test_key.len(), 52);
    assert_eq!(live_key.len(), 52);
}

#[test]
fn test_key_validation() {
    let valid_key = generate_api_key(Environment::Test).unwrap();
    assert!(validate_key_format(&valid_key).is_ok());

    assert!(validate_key_format("invalid_key").is_err());
    assert!(validate_key_format("tronch_sk_test_").is_err());
    assert!(validate_key_format("tronch_sk_live_").is_err());
}

#[test]
fn test_key_uniqueness() {
    let key1 = generate_api_key(Environment::Test).unwrap();
    let key2 = generate_api_key(Environment::Test).unwrap();
    assert_ne!(key1, key2);
} 