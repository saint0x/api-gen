use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use thiserror::Error;
use chrono::{DateTime, Utc};

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Service is unhealthy")]
    Unhealthy,
    #[error("Service is not ready")]
    NotReady,
    #[error("Service is shutting down")]
    ShuttingDown,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub is_ready: bool,
    pub is_shutting_down: bool,
    pub last_check: DateTime<Utc>,
    pub details: Option<String>,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            is_healthy: true,
            is_ready: true,
            is_shutting_down: false,
            last_check: Utc::now(),
            details: None,
        }
    }
}

#[derive(Debug)]
pub struct HealthChecker {
    is_healthy: AtomicBool,
    is_ready: AtomicBool,
    is_shutting_down: AtomicBool,
    last_check: AtomicI64,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            is_healthy: AtomicBool::new(true),
            is_ready: AtomicBool::new(true),
            is_shutting_down: AtomicBool::new(false),
            last_check: AtomicI64::new(Utc::now().timestamp()),
        }
    }

    pub fn check_health(&self) -> Result<HealthStatus, HealthError> {
        if self.is_shutting_down.load(Ordering::Relaxed) {
            return Err(HealthError::ShuttingDown);
        }

        if !self.is_healthy.load(Ordering::Relaxed) {
            return Err(HealthError::Unhealthy);
        }

        if !self.is_ready.load(Ordering::Relaxed) {
            return Err(HealthError::NotReady);
        }

        Ok(HealthStatus {
            is_healthy: self.is_healthy.load(Ordering::Relaxed),
            is_ready: self.is_ready.load(Ordering::Relaxed),
            is_shutting_down: self.is_shutting_down.load(Ordering::Relaxed),
            last_check: DateTime::from_timestamp(self.last_check.load(Ordering::Relaxed), 0)
                .unwrap()
                .with_timezone(&Utc),
            details: None,
        })
    }

    pub fn set_healthy(&self, healthy: bool) {
        self.is_healthy.store(healthy, Ordering::Relaxed);
        self.last_check.store(Utc::now().timestamp(), Ordering::Relaxed);
    }

    pub fn set_ready(&self, ready: bool) {
        self.is_ready.store(ready, Ordering::Relaxed);
        self.last_check.store(Utc::now().timestamp(), Ordering::Relaxed);
    }

    pub fn set_shutting_down(&self, shutting_down: bool) {
        self.is_shutting_down.store(shutting_down, Ordering::Relaxed);
        self.last_check.store(Utc::now().timestamp(), Ordering::Relaxed);
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
} 