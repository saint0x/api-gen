pub mod error;
pub mod generation;
pub mod rate_limit;
pub mod request;
pub mod rotation;
pub mod storage;
pub mod validation;

pub use error::{ApiKeyError, Result};
pub use generation::{generate_api_key, validate_key_format, Environment};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use request::{RequestMetadata, RequestValidator};
pub use rotation::{rotate_key, RotationConfig};
pub use storage::{ApiKeyStorage, InMemoryStorage};
pub use validation::{validate_api_key, ApiKeyMetadata};

// Re-export important types
pub use validation::ApiKeyValidationError;
pub use rotation::KeyRotationError;
