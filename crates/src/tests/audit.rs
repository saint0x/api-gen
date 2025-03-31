use crate::audit::*;
use tokio::time::{sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

#[tokio::test]
async fn test_audit_logging() {
    let (logger, mut rx) = AuditLogger::new(1000, Duration::from_secs(1));
    
    // Start event processing
    let buffer = logger.buffer.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let mut buffer = buffer.write().await;
            buffer.push(event);
        }
    });
    
    let event = AuditEvent {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        event_type: AuditEventType::KeyGenerated,
        key_id: "test_key".to_string(),
        ip_address: "127.0.0.1".to_string(),
        user_agent: "test-agent".to_string(),
        metadata: HashMap::new(),
    };

    logger.log_event(event.clone()).await.unwrap();
    
    // Wait for the event to be processed
    sleep(Duration::from_millis(100)).await;
    
    let events = logger.get_events_for_key("test_key").await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].key_id, "test_key");
}

#[tokio::test]
async fn test_buffer_overflow() {
    let (logger, mut rx) = AuditLogger::new(2, Duration::from_secs(1));
    
    // Start event processing
    let buffer = logger.buffer.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let mut buffer = buffer.write().await;
            if buffer.len() >= 2 {
                return; // Stop processing when buffer is full
            }
            buffer.push(event);
        }
    });
    
    let event = AuditEvent {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        event_type: AuditEventType::KeyGenerated,
        key_id: "test_key".to_string(),
        ip_address: "127.0.0.1".to_string(),
        user_agent: "test-agent".to_string(),
        metadata: HashMap::new(),
    };

    // Add two events (should succeed)
    logger.log_event(event.clone()).await.unwrap();
    logger.log_event(event.clone()).await.unwrap();
    
    // Wait for events to be processed
    sleep(Duration::from_millis(100)).await;
    
    // Try to add a third event (should fail)
    assert!(matches!(
        logger.log_event(event).await,
        Err(AuditError::BufferOverflow)
    ));
}

#[tokio::test]
async fn test_logger_stop() {
    let (logger, _rx) = AuditLogger::new(1000, Duration::from_secs(1));
    
    let event = AuditEvent {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        event_type: AuditEventType::KeyGenerated,
        key_id: "test_key".to_string(),
        ip_address: "127.0.0.1".to_string(),
        user_agent: "test-agent".to_string(),
        metadata: HashMap::new(),
    };

    // Stop the logger
    logger.stop().await;

    // Try to log an event (should fail)
    assert!(matches!(
        logger.log_event(event).await,
        Err(AuditError::LoggerStopped)
    ));
} 