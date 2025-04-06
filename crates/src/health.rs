use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use thiserror::Error;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Service is unhealthy")]
    Unhealthy,
    #[error("Service is not ready")]
    NotReady,
    #[error("Service is shutting down")]
    ShuttingDown,
    #[error("Failed to serialize health response")]
    SerializationError,
    #[error("Failed to send alert")]
    AlertError,
    #[error("Configuration error")]
    ConfigurationError,
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

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub is_ready: bool,
    pub is_healthy: bool,
    pub uptime: i64,
    pub version: String,
}

pub struct HealthEndpoint {
    checker: Arc<HealthChecker>,
    start_time: DateTime<Utc>,
    version: String,
}

impl HealthEndpoint {
    pub fn new(checker: Arc<HealthChecker>, version: String) -> Self {
        Self {
            checker,
            start_time: Utc::now(),
            version,
        }
    }

    pub fn check(&self) -> Result<HealthResponse, HealthError> {
        let health_status = self.checker.check_health()?;
        
        Ok(HealthResponse {
            status: self.status_string(&health_status),
            timestamp: Utc::now(),
            is_ready: health_status.is_ready,
            is_healthy: health_status.is_healthy,
            uptime: (Utc::now() - self.start_time).num_seconds(),
            version: self.version.clone(),
        })
    }

    fn status_string(&self, status: &HealthStatus) -> String {
        if status.is_shutting_down {
            "shutting_down".to_string()
        } else if !status.is_healthy {
            "unhealthy".to_string()
        } else if !status.is_ready {
            "not_ready".to_string()
        } else {
            "healthy".to_string()
        }
    }
}

pub trait AlertNotifier: Send + Sync {
    fn notify(&self, status: &HealthStatus) -> Result<(), HealthError>;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct HealthAlert {
    checker: Arc<HealthChecker>,
    notifier: Box<dyn AlertNotifier>,
    last_notification: AtomicI64,
    min_interval: i64,
}

impl HealthAlert {
    pub fn new(
        checker: Arc<HealthChecker>,
        notifier: Box<dyn AlertNotifier>,
        min_interval: i64,
    ) -> Self {
        Self {
            checker,
            notifier,
            last_notification: AtomicI64::new(0),
            min_interval,
        }
    }

    pub fn check(&self) -> Result<(), HealthError> {
        let now = Utc::now().timestamp();
        let last = self.last_notification.load(Ordering::Relaxed);

        // Check if minimum interval has passed
        if now - last < self.min_interval {
            return Ok(());
        }

        // Check health status - convert errors to status
        let status = match self.checker.check_health() {
            Ok(status) => status,
            Err(HealthError::Unhealthy) => HealthStatus {
                is_healthy: false,
                is_ready: true,
                is_shutting_down: false,
                last_check: Utc::now(),
                details: Some("Service is unhealthy".to_string()),
            },
            Err(HealthError::NotReady) => HealthStatus {
                is_healthy: true,
                is_ready: false,
                is_shutting_down: false,
                last_check: Utc::now(),
                details: Some("Service is not ready".to_string()),
            },
            Err(HealthError::ShuttingDown) => HealthStatus {
                is_healthy: true,
                is_ready: true,
                is_shutting_down: true,
                last_check: Utc::now(),
                details: Some("Service is shutting down".to_string()),
            },
            Err(e) => return Err(e),
        };

        // Only notify if unhealthy, not ready, or shutting down
        if !status.is_healthy || !status.is_ready || status.is_shutting_down {
            self.notifier.notify(&status).map_err(|_| HealthError::AlertError)?;
            self.last_notification.store(now, Ordering::Relaxed);
        }

        Ok(())
    }

    #[cfg(test)]
    pub fn get_notifier(&self) -> &dyn AlertNotifier {
        self.notifier.as_ref()
    }
} 