use crate::rate_limit::*;
use crate::storage::{InMemoryStorage, ApiKeyStorage};
use crate::validation::ApiKeyMetadata;
use crate::generation::Environment;
use std::time::Duration as StdDuration;
use chrono::Duration;
use tokio::time::sleep;
use chrono::Utc;

async fn create_test_storage() -> InMemoryStorage {
    let storage = InMemoryStorage::new();
    let metadata = ApiKeyMetadata {
        created_at: Utc::now(),
        last_used_at: None,
        expires_at: None,
        environment: Environment::Test,
        is_active: true,
        is_revoked: false,
    };
    storage.store_key("test_key", metadata).await.unwrap();
    storage
}

#[tokio::test]
async fn test_rate_limit_basic() {
    let storage = create_test_storage().await;
    let limiter = RateLimiter::new(storage);
    let key = "test_key";
    let result = limiter.check_rate_limit(key).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rate_limit_exceeded() {
    let storage = create_test_storage().await;
    let limiter = RateLimiter::new(storage);
    let key = "test_key";
    
    // Make requests up to the limit
    for _ in 0..100 {
        assert!(limiter.check_rate_limit(key).await.is_ok());
    }

    // Next request should fail
    assert!(matches!(
        limiter.check_rate_limit(key).await,
        Err(RateLimitError::RateLimitExceeded)
    ));
}

#[tokio::test]
async fn test_rate_limit_reset() {
    let storage = create_test_storage().await;
    let limiter = RateLimiter::new(storage);
    let key = "test_key";
    
    // Make requests up to the limit
    for _ in 0..100 {
        assert!(limiter.check_rate_limit(key).await.is_ok());
    }

    // Wait for reset
    sleep(StdDuration::from_secs(61)).await;

    // Should be able to make requests again
    assert!(limiter.check_rate_limit(key).await.is_ok());
}

#[tokio::test]
async fn test_rate_limit_invalid_key() {
    let storage = InMemoryStorage::new();
    let limiter = RateLimiter::new(storage);
    let key = "invalid_key";
    
    assert!(matches!(
        limiter.check_rate_limit(key).await,
        Err(RateLimitError::InvalidKey)
    ));
}

#[tokio::test]
async fn test_rate_limit() {
    let storage = create_test_storage().await;
    let limiter = RateLimiter::new(storage);
    let key = "test_key";

    // First request should succeed
    limiter.check_rate_limit(key).await.unwrap();
    limiter.check_rate_limit(key).await.unwrap();

    // Wait for rate limit to reset
    sleep(StdDuration::from_secs(1)).await;

    // Should succeed again
    limiter.check_rate_limit(key).await.unwrap();
}

#[tokio::test]
async fn test_burst_limit() {
    let storage = create_test_storage().await;
    let config = RateLimitConfig {
        max_requests: 100,
        window: Duration::minutes(1),
        burst_size: 3,
    };
    let mut limiter = RateLimiter::new(storage);
    limiter.set_config(config);
    let key = "test_key";

    // Should allow burst up to size
    for _ in 0..3 {
        limiter.check_rate_limit(key).await.unwrap();
    }

    // Should fail after burst
    assert!(matches!(
        limiter.check_rate_limit(key).await,
        Err(RateLimitError::RateLimitExceeded)
    ));
}