use crate::error::*;

#[test]
fn test_error_creation() {
    // Test basic error creation
    let err = ApiKeyError::InvalidFormat;
    assert_eq!(err.to_string(), "Invalid API key format");

    // Test error with string parameter
    let err = ApiKeyError::InvalidEnvironment("test".to_string());
    assert_eq!(err.to_string(), "Invalid environment: test");

    // Test error with internal error
    let err = ApiKeyError::Internal("test error".to_string());
    assert_eq!(err.to_string(), "Internal error: test error");
}

#[test]
fn test_error_conversion() {
    // Test conversion from HashingError
    let hash_err = HashingError::SaltGenerationError;
    let api_err: ApiKeyError = hash_err.into();
    assert!(matches!(api_err, ApiKeyError::Hashing(_)));

    // Test conversion from StorageError
    let storage_err = StorageError::ConnectionError("test".to_string());
    let api_err: ApiKeyError = storage_err.into();
    assert!(matches!(api_err, ApiKeyError::Storage(_)));

    // Test conversion from ConfigurationError
    let config_err = ConfigurationError::MissingConfig("test".to_string());
    let api_err: ApiKeyError = config_err.into();
    assert!(matches!(api_err, ApiKeyError::Configuration(_)));
}

#[test]
fn test_error_serialization() {
    // Test serialization of ApiKeyError
    let err = ApiKeyError::InvalidFormat;
    let serialized = serde_json::to_string(&err).unwrap();
    let deserialized: ApiKeyError = serde_json::from_str(&serialized).unwrap();
    assert_eq!(err, deserialized);

    // Test serialization of error with string parameter
    let err = ApiKeyError::InvalidEnvironment("test".to_string());
    let serialized = serde_json::to_string(&err).unwrap();
    let deserialized: ApiKeyError = serde_json::from_str(&serialized).unwrap();
    assert_eq!(err, deserialized);
}

#[test]
fn test_error_chaining() {
    // Test error chaining with HashingError
    let hash_err = HashingError::SaltGenerationError;
    let api_err: ApiKeyError = hash_err.into();
    assert!(matches!(api_err, ApiKeyError::Hashing(_)));

    // Test error chaining with StorageError
    let storage_err = StorageError::ConnectionError("test".to_string());
    let api_err: ApiKeyError = storage_err.into();
    assert!(matches!(api_err, ApiKeyError::Storage(_)));
}

#[test]
fn test_error_clone() {
    // Test cloning of ApiKeyError
    let err = ApiKeyError::InvalidFormat;
    let cloned = err.clone();
    assert_eq!(err, cloned);

    // Test cloning of error with string parameter
    let err = ApiKeyError::InvalidEnvironment("test".to_string());
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn test_error_display() {
    // Test display formatting for all error types
    let errors = vec![
        (ApiKeyError::InvalidFormat, "Invalid API key format"),
        (ApiKeyError::NotFound, "API key not found"),
        (ApiKeyError::Revoked, "API key is revoked"),
        (ApiKeyError::Expired, "API key is expired"),
        (ApiKeyError::InvalidEnvironment("test".to_string()), "Invalid environment: test"),
        (ApiKeyError::RateLimitExceeded, "Rate limit exceeded"),
        (ApiKeyError::Internal("test".to_string()), "Internal error: test"),
    ];

    for (err, expected) in errors {
        assert_eq!(err.to_string(), expected);
    }
}

#[test]
fn test_error_equality() {
    // Test equality for simple errors
    let err1 = ApiKeyError::InvalidFormat;
    let err2 = ApiKeyError::InvalidFormat;
    assert_eq!(err1, err2);

    // Test equality for errors with parameters
    let err1 = ApiKeyError::InvalidEnvironment("test".to_string());
    let err2 = ApiKeyError::InvalidEnvironment("test".to_string());
    assert_eq!(err1, err2);

    // Test inequality
    let err1 = ApiKeyError::InvalidFormat;
    let err2 = ApiKeyError::NotFound;
    assert_ne!(err1, err2);
}

#[test]
fn test_error_context() {
    // Test error context preservation
    let hash_err = HashingError::SaltGenerationError;
    let api_err: ApiKeyError = hash_err.into();
    match api_err {
        ApiKeyError::Hashing(e) => assert!(matches!(e, HashingError::SaltGenerationError)),
        _ => panic!("Expected HashingError"),
    }
} 