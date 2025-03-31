use crate::generation::*;

#[test]
fn test_generate_api_key() {
    let key = generate_api_key(Environment::Test).unwrap();
    assert!(key.starts_with("tronch_sk_test_"));
    assert_eq!(key.len(), 52);
}

#[test]
fn test_generate_api_key_live() {
    let key = generate_api_key(Environment::Live).unwrap();
    assert!(key.starts_with("tronch_sk_live_"));
    assert_eq!(key.len(), 52);
}

#[test]
fn test_validate_api_key_format() {
    let key = generate_api_key(Environment::Test).unwrap();
    assert!(validate_key_format(&key, None).is_ok());
}

#[test]
fn test_validate_api_key_format_invalid() {
    let key = "invalid_key";
    assert!(matches!(
        validate_key_format(key, None),
        Err(KeyGenerationError::InvalidFormat)
    ));
}

#[test]
fn test_validate_api_key_format_wrong_prefix() {
    let key = "wrong_prefix_12345678901234567890123456789012345678901";
    assert!(matches!(
        validate_key_format(key, None),
        Err(KeyGenerationError::InvalidFormat)
    ));
}

#[test]
fn test_validate_api_key_format_wrong_length() {
    let key = "tronch_sk_test_123";
    assert!(matches!(
        validate_key_format(key, None),
        Err(KeyGenerationError::InvalidFormat)
    ));
}

#[test]
fn test_validate_api_key_format_invalid_chars() {
    let key = "tronch_sk_test_123456789012345678901234567890123456789!";
    assert!(matches!(
        validate_key_format(key, None),
        Err(KeyGenerationError::InvalidFormat)
    ));
}