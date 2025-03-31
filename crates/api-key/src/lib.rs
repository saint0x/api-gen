pub mod generation;
pub mod validation;
pub mod storage;
pub mod rotation;
pub mod rate_limit;

pub use generation::{generate_api_key, validate_key_format, Environment};
pub use validation::{validate_api_key, ApiKeyMetadata};
pub use storage::{ApiKeyStorage, InMemoryStorage};
pub use rotation::{rotate_key, RotationConfig};
pub use rate_limit::{RateLimiter, RateLimitConfig, RateLimitError};

// Re-export important types
pub use validation::ApiKeyValidationError;
pub use rotation::KeyRotationError;
