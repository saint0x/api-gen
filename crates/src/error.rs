use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::request::RequestValidationError;

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiKeyError {
    #[error("Invalid API key format")]
    InvalidFormat,
    #[error("API key not found")]
    NotFound,
    #[error("API key is revoked")]
    Revoked,
    #[error("API key is expired")]
    Expired,
    #[error("Invalid environment: {0}")]
    InvalidEnvironment(String),
    #[error("Request validation error: {0}")]
    RequestValidation(#[from] RequestValidationError),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Hashing error: {0}")]
    Hashing(#[from] HashingError),
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
    #[error("Audit error: {0}")]
    Audit(#[from] AuditError),
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq)]
pub enum HashingError {
    #[error("Failed to generate salt")]
    SaltGenerationError,
    #[error("Invalid hash format")]
    InvalidHashFormat,
    #[error("Hash verification failed")]
    VerificationFailed,
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Migration error: {0}")]
    MigrationError(String),
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigurationError {
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    #[error("Environment error: {0}")]
    EnvironmentError(String),
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditError {
    #[error("Failed to write audit log: {0}")]
    WriteError(String),
    #[error("Buffer overflow")]
    BufferOverflow,
    #[error("Logger stopped")]
    LoggerStopped,
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationError {
    #[error("Invalid key format")]
    InvalidFormat,
    #[error("Key not found")]
    NotFound,
    #[error("Key revoked")]
    Revoked,
    #[error("Key expired")]
    Expired,
    #[error("Invalid environment: {0}")]
    InvalidEnvironment(String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Invalid IP address")]
    InvalidIpAddress,
}

pub type Result<T> = std::result::Result<T, ApiKeyError>; 