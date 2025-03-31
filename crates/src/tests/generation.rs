use crate::generation::{generate_api_key, validate_key_format, Environment};

#[test]
fn test_generate_api_key() {
    let (key, metadata) = generate_api_key(Environment::Test).unwrap();
    assert!(key.starts_with("tronch_sk_test_"));
    assert_eq!(key.len(), 52);
    assert!(metadata.verify_key(&key).unwrap());
    assert_eq!(metadata.environment, Environment::Test);
}

#[test]
fn test_generate_api_key_live() {
    let (key, metadata) = generate_api_key(Environment::Live).unwrap();
    assert!(key.starts_with("tronch_sk_live_"));
    assert_eq!(key.len(), 52);
    assert!(metadata.verify_key(&key).unwrap());
    assert_eq!(metadata.environment, Environment::Live);
}

#[test]
fn test_validate_api_key_format() {
    let (key, _) = generate_api_key(Environment::Test).unwrap();
    assert!(validate_key_format(&key, None).is_ok());
    assert!(validate_key_format(&key, Some(Environment::Test)).is_ok());
}

#[test]
fn test_validate_api_key_format_wrong_prefix() {
    assert!(validate_key_format("invalid_prefix_20240101abcdefghijklmnopqrstuvwxyz", None).is_err());
}

#[test]
fn test_validate_api_key_format_wrong_length() {
    assert!(validate_key_format("tronch_sk_test_20240101abcdefghijklmnopqrstuvwxyz_too_long", None).is_err());
    assert!(validate_key_format("tronch_sk_test_too_short", None).is_err());
}

#[test]
fn test_validate_api_key_format_invalid_chars() {
    assert!(validate_key_format("tronch_sk_test_20240101!@#$%^&*()_+abcdefghijklm", None).is_err());
}

#[test]
fn test_validate_api_key_format_invalid() {
    let (key, _) = generate_api_key(Environment::Test).unwrap();
    assert!(validate_key_format(&key, Some(Environment::Live)).is_err());
}