use thiserror::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Duration, Utc};
use crate::storage::ApiKeyStorage;

#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid rate limit configuration")]
    InvalidConfig,
    #[error("Storage error")]
    StorageError,
    #[error("Invalid API key")]
    InvalidKey,
}

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    /// Time window for rate limiting (e.g., 1 minute, 1 hour)
    pub window: Duration,
    /// Maximum burst size allowed
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::minutes(1),
            burst_size: 100,
        }
    }
}

/// Represents a rate limit counter for a specific key
#[derive(Debug, Clone)]
struct RateLimitCounter {
    /// Current count of requests in the window
    count: u32,
    /// When the current window started
    window_start: DateTime<Utc>,
    /// When the counter was last updated
    last_updated: DateTime<Utc>,
    /// Current burst count
    burst_count: u32,
    /// When the current burst window started
    burst_start: DateTime<Utc>,
}

impl RateLimitCounter {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            count: 0,
            window_start: now,
            last_updated: now,
            burst_count: 0,
            burst_start: now,
        }
    }

    fn is_window_expired(&self, window: Duration) -> bool {
        Utc::now() - self.window_start > window
    }

    fn is_burst_expired(&self, burst_window: Duration) -> bool {
        Utc::now() - self.burst_start > burst_window
    }

    fn reset(&mut self) {
        let now = Utc::now();
        self.count = 0;
        self.window_start = now;
        self.last_updated = now;
        self.burst_count = 0;
        self.burst_start = now;
    }

    fn increment(&mut self) {
        let now = Utc::now();
        self.count += 1;
        self.last_updated = now;

        // Reset burst counter if burst window has expired
        if self.is_burst_expired(Duration::seconds(1)) {
            self.burst_count = 0;
            self.burst_start = now;
        }
        self.burst_count += 1;
    }
}

/// In-memory storage for rate limit counters
#[derive(Debug, Default)]
struct InMemoryRateLimitStorage {
    counters: Arc<RwLock<HashMap<String, RateLimitCounter>>>,
}

impl InMemoryRateLimitStorage {
    fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Rate limiter implementation
#[derive(Debug)]
pub struct RateLimiter<S: ApiKeyStorage> {
    storage: InMemoryRateLimitStorage,
    config: RateLimitConfig,
    key_storage: S,
}

impl<S: ApiKeyStorage> RateLimiter<S> {
    pub fn new(key_storage: S) -> Self {
        Self {
            storage: InMemoryRateLimitStorage::new(),
            config: RateLimitConfig::default(),
            key_storage,
        }
    }

    pub fn set_config(&mut self, config: RateLimitConfig) {
        self.config = config;
    }

    /// Check if a request should be allowed based on rate limits
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        // First verify the key exists
        if self.key_storage.get_metadata(key).await.is_err() {
            return Err(RateLimitError::InvalidKey);
        }

        let mut counters = self.storage.counters.write().await;
        let counter = counters.entry(key.to_string()).or_insert_with(RateLimitCounter::new);

        // Reset counter if window has expired
        if counter.is_window_expired(self.config.window) {
            counter.reset();
        }

        // Check if we've exceeded the rate limit
        if counter.count >= self.config.max_requests {
            return Err(RateLimitError::RateLimitExceeded);
        }

        // Check burst limit
        if counter.burst_count >= self.config.burst_size {
            return Err(RateLimitError::RateLimitExceeded);
        }

        // Increment counter
        counter.increment();
        Ok(())
    }

    /// Get current rate limit status for a key
    pub async fn get_status(&self, key: &str) -> Option<(u32, DateTime<Utc>)> {
        let counters = self.storage.counters.read().await;
        counters.get(key).map(|counter| (counter.count, counter.window_start))
    }

    /// Reset rate limit counter for a key
    pub async fn reset_counter(&self, key: &str) {
        let mut counters = self.storage.counters.write().await;
        if let Some(counter) = counters.get_mut(key) {
            counter.reset();
        }
    }
} 