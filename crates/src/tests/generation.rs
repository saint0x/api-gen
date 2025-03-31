use tronch::{
    Environment,
    generate_api_key,
    validate_key_format,
};

mod common;

#[test]
fn test_key_generation() {
    let test_key = generate_api_key(Environment::Test);
    let live_key = generate_api_key(Environment::Live);

    assert!(test_key.starts_with("tronch_sk_test_"));
    assert!(live_key.starts_with("tronch_sk_live_"));
    assert_eq!(test_key.len(), 52);
    assert_eq!(live_key.len(), 52);
}

#[test]
fn test_key_validation() {
    let valid_key = generate_api_key(Environment::Test);
    assert!(validate_key_format(&valid_key));

    assert!(!validate_key_format("invalid_key"));
    assert!(!validate_key_format("tronch_sk_test_"));
    assert!(!validate_key_format("tronch_sk_live_"));
}

#[test]
fn test_key_uniqueness() {
    let key1 = generate_api_key(Environment::Test);
    let key2 = generate_api_key(Environment::Test);
    assert_ne!(key1, key2);
} 