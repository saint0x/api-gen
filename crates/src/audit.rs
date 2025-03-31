use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc::Sender;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Failed to serialize audit log: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Failed to write to audit log: {0}")]
    WriteError(String),
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Buffer overflow")]
    BufferOverflow,
    #[error("Logger is stopped")]
    LoggerStopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub key_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    KeyGenerated,
    KeyRevoked,
    KeyRotated,
    KeyValidated,
    KeyInvalidated,
    RateLimitExceeded,
    RequestBlocked,
}

pub struct AuditLogger {
    pub(crate) buffer: Arc<RwLock<Vec<AuditEvent>>>,
    buffer_size: usize,
    flush_interval: Duration,
    tx: Sender<AuditEvent>,
    is_running: Arc<RwLock<bool>>,
}

impl AuditLogger {
    pub fn new(buffer_size: usize, flush_interval: Duration) -> (Self, mpsc::Receiver<AuditEvent>) {
        let (tx, rx) = mpsc::channel(1000);
        let logger = Self {
            buffer: Arc::new(RwLock::new(Vec::with_capacity(buffer_size))),
            buffer_size,
            flush_interval,
            tx,
            is_running: Arc::new(RwLock::new(true)),
        };
        (logger, rx)
    }

    pub async fn log_event(&self, event: AuditEvent) -> Result<(), AuditError> {
        // Check if logger is running
        if !*self.is_running.read().await {
            return Err(AuditError::LoggerStopped);
        }

        // Check buffer size before sending
        let buffer = self.buffer.read().await;
        if buffer.len() >= self.buffer_size {
            return Err(AuditError::BufferOverflow);
        }

        // Send event through channel
        self.tx.send(event).await.map_err(|e| AuditError::WriteError(e.to_string()))?;
        Ok(())
    }

    pub async fn process_events(&self, mut rx: mpsc::Receiver<AuditEvent>) {
        while let Some(event) = rx.recv().await {
            let mut buffer = self.buffer.write().await;
            if buffer.len() >= self.buffer_size {
                // Buffer is full, flush it
                buffer.clear();
            }
            buffer.push(event);
        }
    }

    pub async fn flush(&self) -> Result<(), AuditError> {
        let mut buffer = self.buffer.write().await;
        if !buffer.is_empty() {
            // Here you would implement your actual storage logic
            // For example, writing to a file, sending to a logging service, etc.
            buffer.clear();
        }
        Ok(())
    }

    pub async fn get_events_for_key(&self, key_id: &str) -> Result<Vec<AuditEvent>, AuditError> {
        let buffer = self.buffer.read().await;
        Ok(buffer.iter()
            .filter(|event| event.key_id == key_id)
            .cloned()
            .collect())
    }

    pub async fn get_events_by_type(&self, event_type: AuditEventType) -> Result<Vec<AuditEvent>, AuditError> {
        let buffer = self.buffer.read().await;
        Ok(buffer.iter()
            .filter(|event| event.event_type == event_type)
            .cloned()
            .collect())
    }

    pub async fn start_periodic_flush(&self) {
        let buffer = self.buffer.clone();
        let flush_interval = self.flush_interval;
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            while *is_running.read().await {
                sleep(flush_interval).await;
                let mut buffer = buffer.write().await;
                if !buffer.is_empty() {
                    buffer.clear();
                }
            }
        });
    }

    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
    }
} 