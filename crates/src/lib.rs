pub mod error;
pub mod generation;
pub mod rate_limit;
pub mod request;
pub mod rotation;
pub mod storage;
pub mod validation;
pub mod audit;

pub use error::{ApiKeyError, Result};
pub use generation::{generate_api_key, validate_key_format, Environment, KeyGenerationError};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use request::{RequestMetadata, RequestValidator};
pub use rotation::{rotate_key, RotationConfig, KeyRotationError};
pub use storage::{ApiKeyStorage, InMemoryStorage, StorageError};
pub use validation::{validate_api_key, ApiKeyMetadata, ApiKeyValidationError};

// Re-export important types
pub use audit::{AuditLogger, AuditEvent, AuditEventType, AuditError};
