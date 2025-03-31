use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use chrono::{Duration, Utc};
use dashmap::DashMap;
use thiserror::Error;
use crate::storage::ApiKeyStorage;
use async_trait::async_trait;

/// Trait for providing time, allowing for test mocking
pub trait TimeProvider: Send + Sync + std::fmt::Debug {
    fn current_time(&self) -> i64;
}

#[derive(Debug)]
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn current_time(&self) -> i64 {
        Utc::now().timestamp()
    }
}

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window
    pub max_requests: i64,
    /// Time window for rate limiting
    pub window: Duration,
    /// Maximum number of requests allowed in a burst (per second)
    pub burst_size: i64,
    /// Rate at which tokens are refilled (tokens per second)
    pub refill_rate: i64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::minutes(1),
            burst_size: 10,
            refill_rate: 10, // 10 tokens per second
        }
    }
}

/// Internal state for rate limiting
#[derive(Debug)]
pub struct RateLimitState {
    // Fixed Window Counter
    window_start: AtomicI64,
    request_count: AtomicI64,
    
    // Token Bucket
    tokens: AtomicI64,
    last_refill: AtomicI64,
}

impl RateLimitState {
    fn new(now: i64) -> Self {
        Self {
            window_start: AtomicI64::new(now),
            request_count: AtomicI64::new(0),
            tokens: AtomicI64::new(0),
            last_refill: AtomicI64::new(now),
        }
    }

    fn refill_tokens(&self, now: i64, refill_rate: i64, burst_size: i64) -> i64 {
        let elapsed = now - self.last_refill.load(Ordering::Relaxed);
        let new_tokens = elapsed * refill_rate;
        let current = self.tokens.load(Ordering::Relaxed);
        
        // When current is negative, we need more tokens to get back to positive
        let updated = if current < 0 {
            // We need abs(current) tokens just to get back to 0
            // Then we can add any remaining tokens up to burst_size
            let tokens_to_zero = -current;
            if new_tokens <= tokens_to_zero {
                current + new_tokens // Still negative or zero
            } else {
                // We have tokens remaining after getting to zero
                let remaining_tokens = new_tokens - tokens_to_zero;
                remaining_tokens.min(burst_size)
            }
        } else {
            (current + new_tokens).min(burst_size)
        };
        
        self.tokens.store(updated, Ordering::Relaxed);
        self.last_refill.store(now, Ordering::Relaxed);
        updated
    }

    fn check_window(&self, now: i64, window_size: i64, max_requests: i64) -> bool {
        let window_start = self.window_start.load(Ordering::Relaxed);
        if now - window_start >= window_size {
            self.window_start.store(now, Ordering::Relaxed);
            self.request_count.store(0, Ordering::Relaxed);
        }
        self.request_count.load(Ordering::Relaxed) < max_requests
    }

    fn increment_counters(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.tokens.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Storage trait for rate limiting
#[async_trait]
pub trait RateLimitStorage: Send + Sync + std::fmt::Debug {
    async fn get_metadata(&self, key: &str) -> Result<(), RateLimitError>;
    async fn get_state(&self, key: &str) -> Option<Arc<RateLimitState>>;
    async fn set_state(&self, key: &str, state: Arc<RateLimitState>);
}

/// In-memory storage implementation
#[derive(Debug)]
pub struct InMemoryRateLimitStorage {
    api_storage: Arc<dyn ApiKeyStorage>,
    states: DashMap<String, Arc<RateLimitState>>,
}

impl InMemoryRateLimitStorage {
    pub fn new(api_storage: Arc<dyn ApiKeyStorage>) -> Self {
        Self {
            api_storage,
            states: DashMap::new(),
        }
    }
}

#[async_trait]
impl RateLimitStorage for InMemoryRateLimitStorage {
    async fn get_metadata(&self, key: &str) -> Result<(), RateLimitError> {
        self.api_storage.get_metadata(key).await.map_err(|_| RateLimitError::InvalidKey)?;
        Ok(())
    }

    async fn get_state(&self, key: &str) -> Option<Arc<RateLimitState>> {
        self.states.get(key).map(|entry| entry.value().clone())
    }

    async fn set_state(&self, key: &str, state: Arc<RateLimitState>) {
        self.states.insert(key.to_string(), state);
    }
}

/// Main rate limiter implementation
#[derive(Debug)]
pub struct RateLimiter<S: RateLimitStorage> {
    storage: S,
    config: Arc<RateLimitConfig>,
    time_provider: Arc<dyn TimeProvider>,
}

impl<S: RateLimitStorage> RateLimiter<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            config: Arc::new(RateLimitConfig::default()),
            time_provider: Arc::new(SystemTimeProvider),
        }
    }

    pub fn with_time_provider(storage: S, time_provider: Arc<dyn TimeProvider>) -> Self {
        Self {
            storage,
            config: Arc::new(RateLimitConfig::default()),
            time_provider,
        }
    }

    pub fn set_config(&mut self, config: RateLimitConfig) {
        self.config = Arc::new(config);
    }

    async fn get_or_create_state(&self, key: &str) -> Arc<RateLimitState> {
        if let Some(state) = self.storage.get_state(key).await {
            state
        } else {
            let current_time = self.time_provider.current_time();
            let state = Arc::new(RateLimitState::new(current_time));
            state.tokens.store(self.config.burst_size, Ordering::Relaxed);
            self.storage.set_state(key, state.clone()).await;
            state
        }
    }

    /// Check if a request should be allowed based on rate limits
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        // First verify the key exists
        self.storage.get_metadata(key).await?;

        let current_time = self.time_provider.current_time();
        let state = self.get_or_create_state(key).await;

        // Check fixed window rate limit
        if !state.check_window(
            current_time,
            self.config.window.num_seconds(),
            self.config.max_requests,
        ) {
            return Err(RateLimitError::RateLimitExceeded);
        }

        // Check token bucket burst limit
        let tokens = state.refill_tokens(
            current_time,
            self.config.refill_rate,
            self.config.burst_size,
        );

        if tokens < 1 {
            return Err(RateLimitError::RateLimitExceeded);
        }

        // Update counters
        state.increment_counters();

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid API key")]
    InvalidKey,
} 