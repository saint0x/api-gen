use thiserror::Error;

use crate::request::RequestValidationError;

#[derive(Debug, Error)]
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
}

pub type Result<T> = std::result::Result<T, ApiKeyError>; 