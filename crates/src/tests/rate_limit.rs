use tronch_api_key::{
    rate_limit::{RateLimitError, RateLimiter},
    InMemoryStorage,
};

mod common;
use common::create_test_storage;

#[tokio::test]
async fn test_rate_limit_basic() {
    let (storage, key, _) = create_test_storage().await;
    let limiter = RateLimiter::new(storage);
    let result = limiter.check_rate_limit(&key).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rate_limit_exceeded() {
    let (storage, key, _) = create_test_storage().await;
    let limiter = RateLimiter::new(storage);
    
    // Make requests up to the limit
    for _ in 0..100 {
        assert!(limiter.check_rate_limit(&key).await.is_ok());
    }

    // Next request should fail
    assert!(matches!(
        limiter.check_rate_limit(&key).await,
        Err(RateLimitError::RateLimitExceeded)
    ));
}

#[tokio::test]
async fn test_rate_limit_reset() {
    let (storage, key, _) = create_test_storage().await;
    let limiter = RateLimiter::new(storage);
    
    // Make requests up to the limit
    for _ in 0..100 {
        assert!(limiter.check_rate_limit(&key).await.is_ok());
    }

    // Wait for reset
    tokio::time::sleep(tokio::time::Duration::from_secs(61)).await;

    // Should be able to make requests again
    assert!(limiter.check_rate_limit(&key).await.is_ok());
}

#[tokio::test]
async fn test_rate_limit_invalid_key() {
    let storage = InMemoryStorage::new();
    let limiter = RateLimiter::new(storage);
    
    assert!(matches!(
        limiter.check_rate_limit("invalid_key").await,
        Err(RateLimitError::InvalidKey)
    ));
} 