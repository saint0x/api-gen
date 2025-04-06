use crate::health::*;
use chrono::Utc;
use std::thread::sleep;
use std::time::Duration;
use std::sync::Arc;
use std::any::Any;

#[derive(Debug)]
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
    fn notify(&self, _status: &HealthStatus) -> Result<(), HealthError> {
        self.notifications.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Helper function to get notification count from AlertNotifier
fn get_notification_count(notifier: &dyn AlertNotifier) -> usize {
    notifier.as_any()
        .downcast_ref::<MockNotifier>()
        .expect("Failed to downcast to MockNotifier")
        .notification_count()
}

#[test]
fn test_health_checker_initialization() {
    let checker = HealthChecker::new();
    let status = checker.check_health().unwrap();
    
    assert!(status.is_healthy);
    assert!(status.is_ready);
    assert!(!status.is_shutting_down);
    assert!(status.last_check <= Utc::now());
}

#[test]
fn test_health_status_transitions() {
    let checker = HealthChecker::new();
    
    // Test unhealthy state
    checker.set_healthy(false);
    assert!(matches!(checker.check_health(), Err(HealthError::Unhealthy)));
    
    // Test not ready state
    checker.set_healthy(true);
    checker.set_ready(false);
    assert!(matches!(checker.check_health(), Err(HealthError::NotReady)));
    
    // Test shutting down state
    checker.set_ready(true);
    checker.set_shutting_down(true);
    assert!(matches!(checker.check_health(), Err(HealthError::ShuttingDown)));
    
    // Test recovery
    checker.set_shutting_down(false);
    let status = checker.check_health().unwrap();
    assert!(status.is_healthy);
    assert!(status.is_ready);
    assert!(!status.is_shutting_down);
}

#[test]
fn test_health_status_default() {
    let status = HealthStatus::default();
    
    assert!(status.is_healthy);
    assert!(status.is_ready);
    assert!(!status.is_shutting_down);
    assert!(status.last_check <= Utc::now());
    assert!(status.details.is_none());
}

#[test]
fn test_health_checker_default() {
    let checker = HealthChecker::default();
    let status = checker.check_health().unwrap();
    
    assert!(status.is_healthy);
    assert!(status.is_ready);
    assert!(!status.is_shutting_down);
    assert!(status.last_check <= Utc::now());
}

#[test]
fn test_last_check_updates() {
    let checker = HealthChecker::new();
    let initial_check = checker.check_health().unwrap().last_check;
    
    // Wait for 2 seconds to ensure timestamp changes
    sleep(Duration::from_secs(2));
    
    // Update any state to trigger last_check update
    checker.set_healthy(true);
    let new_check = checker.check_health().unwrap().last_check;
    
    assert!(new_check > initial_check);
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
    
    match endpoint.check() {
        Ok(_) => panic!("Expected unhealthy error"),
        Err(HealthError::Unhealthy) => (),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_health_endpoint_not_ready() {
    let checker = Arc::new(HealthChecker::new());
    checker.set_ready(false);
    let endpoint = HealthEndpoint::new(checker, "1.0.0".to_string());
    
    match endpoint.check() {
        Ok(_) => panic!("Expected not ready error"),
        Err(HealthError::NotReady) => (),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_health_alert_notification() {
    let checker = Arc::new(HealthChecker::new());
    let notifier = Box::new(MockNotifier::new());
    let alert = HealthAlert::new(checker.clone(), notifier, 0);

    // Initial check should be healthy
    assert!(matches!(alert.check(), Ok(())));
    assert_eq!(get_notification_count(alert.get_notifier()), 0);

    // Set unhealthy and check
    checker.set_healthy(false);
    assert!(matches!(alert.check(), Ok(())));
    assert_eq!(get_notification_count(alert.get_notifier()), 1);

    // Set back to healthy and check
    checker.set_healthy(true);
    assert!(matches!(alert.check(), Ok(())));
    assert_eq!(get_notification_count(alert.get_notifier()), 1); // No new notification

    // Set not ready and check
    checker.set_ready(false);
    assert!(matches!(alert.check(), Ok(())));
    assert_eq!(get_notification_count(alert.get_notifier()), 2);

    // Set back to ready and check
    checker.set_ready(true);
    assert!(matches!(alert.check(), Ok(())));
    assert_eq!(get_notification_count(alert.get_notifier()), 2); // No new notification
}

#[test]
fn test_health_alert_minimum_interval() {
    let checker = Arc::new(HealthChecker::new());
    let notifier = Box::new(MockNotifier::new());
    let alert = HealthAlert::new(checker.clone(), notifier, 60); // 60 second interval

    // Set unhealthy and check
    checker.set_healthy(false);
    assert!(matches!(alert.check(), Ok(())));
    assert_eq!(get_notification_count(alert.get_notifier()), 1);

    // Check again immediately - should not notify due to interval
    assert!(matches!(alert.check(), Ok(())));
    assert_eq!(get_notification_count(alert.get_notifier()), 1);
} 