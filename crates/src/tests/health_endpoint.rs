use std::sync::Arc;
use chrono::Utc;
use std::thread::sleep;
use std::time::Duration;

use crate::health::HealthChecker;
use crate::health_endpoint::{HealthEndpoint, HealthAlert, AlertNotifier, HealthAlertError};

struct MockNotifier {
    notifications: std::sync::atomic::AtomicUsize,
}

impl MockNotifier {
    fn new() -> Self {
        Self {
            notifications: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn notification_count(&self) -> usize {
        self.notifications.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl AlertNotifier for MockNotifier {
    fn notify(&self, _status: &crate::health::HealthStatus) -> Result<(), HealthAlertError> {
        self.notifications.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn test_health_endpoint_healthy() {
    let checker = Arc::new(HealthChecker::new());
    let endpoint = HealthEndpoint::new(checker, "1.0.0".to_string());
    
    let response = endpoint.check().unwrap();
    assert_eq!(response.status, "healthy");
    assert!(response.is_healthy);
    assert!(response.is_ready);
    assert_eq!(response.version, "1.0.0");
    assert!(response.uptime >= 0);
}

#[test]
fn test_health_endpoint_unhealthy() {
    let checker = Arc::new(HealthChecker::new());
    checker.set_healthy(false);
    let endpoint = HealthEndpoint::new(checker, "1.0.0".to_string());
    
    let response = endpoint.check().unwrap();
    assert_eq!(response.status, "unhealthy");
    assert!(!response.is_healthy);
}

#[test]
fn test_health_endpoint_not_ready() {
    let checker = Arc::new(HealthChecker::new());
    checker.set_ready(false);
    let endpoint = HealthEndpoint::new(checker, "1.0.0".to_string());
    
    let response = endpoint.check().unwrap();
    assert_eq!(response.status, "not_ready");
    assert!(!response.is_ready);
}

#[test]
fn test_health_alert_notification() {
    let checker = Arc::new(HealthChecker::new());
    let notifier = Box::new(MockNotifier::new());
    let alert = HealthAlert::new(checker.clone(), notifier, 1);
    
    // Should not trigger alert when healthy
    alert.check().unwrap();
    assert_eq!(alert.notifier.as_any().downcast_ref::<MockNotifier>().unwrap().notification_count(), 0);
    
    // Should trigger alert when unhealthy
    checker.set_healthy(false);
    alert.check().unwrap();
    assert_eq!(alert.notifier.as_any().downcast_ref::<MockNotifier>().unwrap().notification_count(), 1);
}

#[test]
fn test_health_alert_minimum_interval() {
    let checker = Arc::new(HealthChecker::new());
    checker.set_healthy(false);
    let notifier = Box::new(MockNotifier::new());
    let alert = HealthAlert::new(checker, notifier, 2); // 2 second minimum interval
    
    // First alert should trigger
    alert.check().unwrap();
    assert_eq!(alert.notifier.as_any().downcast_ref::<MockNotifier>().unwrap().notification_count(), 1);
    
    // Second alert should not trigger immediately
    alert.check().unwrap();
    assert_eq!(alert.notifier.as_any().downcast_ref::<MockNotifier>().unwrap().notification_count(), 1);
    
    // Wait for interval
    sleep(Duration::from_secs(2));
    
    // Third alert should trigger after interval
    alert.check().unwrap();
    assert_eq!(alert.notifier.as_any().downcast_ref::<MockNotifier>().unwrap().notification_count(), 2);
} 