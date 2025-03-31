use crate::rate_limit::*;
use crate::storage::{InMemoryStorage, ApiKeyStorage};
use crate::validation::ApiKeyMetadata;
use crate::generation::Environment;
use chrono::Duration;
use std::sync::atomic::AtomicI64;
use std::sync::Arc;

/// Mock time provider for testing
#[derive(Debug)]
struct MockTimeProvider {
    current: AtomicI64,
}

impl MockTimeProvider {
    fn new(initial_time: i64) -> Self {
        Self {
            current: AtomicI64::new(initial_time),
        }
    }

    fn advance(&self, seconds: i64) {
        self.current.fetch_add(seconds, std::sync::atomic::Ordering::SeqCst);
    }
}

impl TimeProvider for MockTimeProvider {
    fn current_time(&self) -> i64 {
        self.current.load(std::sync::atomic::Ordering::SeqCst)
    }
}

async fn create_test_storage() -> InMemoryStorage {
    let storage = InMemoryStorage::new();
    
    // Initialize test keys
    let test_keys = ["test_key", "key1", "key2"];
    for key in test_keys {
        let metadata = ApiKeyMetadata::new(Environment::Test, key).unwrap();
        storage.store_key(key, metadata).await.unwrap();
    }
    
    storage
}

#[tokio::test]
async fn test_basic_rate_limit() {
    let storage = Arc::new(create_test_storage().await);
    let rate_limit_storage = InMemoryRateLimitStorage::new(storage.clone());
    let time_provider = Arc::new(MockTimeProvider::new(1000));
    let mut limiter = RateLimiter::with_time_provider(rate_limit_storage, time_provider.clone());

    let config = RateLimitConfig {
        max_requests: 2,
        window: Duration::seconds(60),
        burst_size: 2,
        refill_rate: 2,
    };
    limiter.set_config(config);

    // First request should succeed
    assert!(limiter.check_rate_limit("test_key").await.is_ok());

    // Second request should succeed
    assert!(limiter.check_rate_limit("test_key").await.is_ok());

    // Third request should fail
    assert!(matches!(
        limiter.check_rate_limit("test_key").await,
        Err(RateLimitError::RateLimitExceeded)
    ));
}

#[tokio::test]
async fn test_window_reset() {
    let storage = Arc::new(create_test_storage().await);
    let rate_limit_storage = InMemoryRateLimitStorage::new(storage.clone());
    let time_provider = Arc::new(MockTimeProvider::new(1000));
    let mut limiter = RateLimiter::with_time_provider(rate_limit_storage, time_provider.clone());

    let config = RateLimitConfig {
        max_requests: 2,
        window: Duration::seconds(60),
        burst_size: 2,
        refill_rate: 2,
    };
    limiter.set_config(config);

    // Use up the rate limit
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    assert!(matches!(
        limiter.check_rate_limit("test_key").await,
        Err(RateLimitError::RateLimitExceeded)
    ));

    // Advance time past the window
    time_provider.advance(61);

    // Should be able to make requests again
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    assert!(matches!(
        limiter.check_rate_limit("test_key").await,
        Err(RateLimitError::RateLimitExceeded)
    ));
}

#[tokio::test]
async fn test_burst_limit() {
    let storage = Arc::new(create_test_storage().await);
    let rate_limit_storage = InMemoryRateLimitStorage::new(storage.clone());
    let time_provider = Arc::new(MockTimeProvider::new(1000));
    let mut limiter = RateLimiter::with_time_provider(rate_limit_storage, time_provider.clone());

    let config = RateLimitConfig {
        max_requests: 100, // High enough to not interfere
        window: Duration::seconds(60),
        burst_size: 2, // Only allow 2 tokens max
        refill_rate: 1, // 1 token per second
    };
    limiter.set_config(config);

    // First request should succeed (2 tokens -> 1 token)
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    
    // Second request should succeed (1 token -> 0 tokens)
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    
    // Third request should fail (0 tokens)
    assert!(matches!(
        limiter.check_rate_limit("test_key").await,
        Err(RateLimitError::RateLimitExceeded)
    ));

    // Wait 1 second (should get 1 token)
    time_provider.advance(1);

    // Should be able to make one request (1 token -> 0 tokens)
    assert!(limiter.check_rate_limit("test_key").await.is_ok());

    // Should fail again (0 tokens)
    assert!(matches!(
        limiter.check_rate_limit("test_key").await,
        Err(RateLimitError::RateLimitExceeded)
    ));
}

#[tokio::test]
async fn test_token_refill() {
    let storage = Arc::new(create_test_storage().await);
    let rate_limit_storage = InMemoryRateLimitStorage::new(storage.clone());
    let time_provider = Arc::new(MockTimeProvider::new(1000));
    let mut limiter = RateLimiter::with_time_provider(rate_limit_storage, time_provider.clone());

    let config = RateLimitConfig {
        max_requests: 100,
        window: Duration::seconds(60),
        burst_size: 2,
        refill_rate: 1, // 1 token per second
    };
    limiter.set_config(config);

    // Use up initial tokens
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    assert!(matches!(
        limiter.check_rate_limit("test_key").await,
        Err(RateLimitError::RateLimitExceeded)
    ));

    // Wait for 1.5 seconds (should get 1.5 tokens)
    time_provider.advance(1);

    // Should be able to make one more request
    assert!(limiter.check_rate_limit("test_key").await.is_ok());
    assert!(matches!(
        limiter.check_rate_limit("test_key").await,
        Err(RateLimitError::RateLimitExceeded)
    ));
}

#[tokio::test]
async fn test_invalid_key() {
    let storage = Arc::new(create_test_storage().await);
    let rate_limit_storage = InMemoryRateLimitStorage::new(storage.clone());
    let time_provider = Arc::new(MockTimeProvider::new(1000));
    let limiter = RateLimiter::with_time_provider(rate_limit_storage, time_provider);

    // Request with invalid key should fail
    assert!(matches!(
        limiter.check_rate_limit("invalid_key").await,
        Err(RateLimitError::InvalidKey)
    ));
}

#[tokio::test]
async fn test_multiple_keys() {
    let storage = Arc::new(create_test_storage().await);
    let rate_limit_storage = InMemoryRateLimitStorage::new(storage.clone());
    let time_provider = Arc::new(MockTimeProvider::new(1000));
    let mut limiter = RateLimiter::with_time_provider(rate_limit_storage, time_provider);

    let config = RateLimitConfig {
        max_requests: 2,
        window: Duration::seconds(60),
        burst_size: 2,
        refill_rate: 2,
    };
    limiter.set_config(config);

    // Use up rate limit for first key
    assert!(limiter.check_rate_limit("key1").await.is_ok());
    assert!(limiter.check_rate_limit("key1").await.is_ok());
    assert!(matches!(
        limiter.check_rate_limit("key1").await,
        Err(RateLimitError::RateLimitExceeded)
    ));

    // Second key should still have full rate limit
    assert!(limiter.check_rate_limit("key2").await.is_ok());
    assert!(limiter.check_rate_limit("key2").await.is_ok());
    assert!(matches!(
        limiter.check_rate_limit("key2").await,
        Err(RateLimitError::RateLimitExceeded)
    ));
} 