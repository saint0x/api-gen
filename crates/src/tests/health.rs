use crate::health::*;
use chrono::Utc;
use std::thread::sleep;
use std::time::Duration;

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