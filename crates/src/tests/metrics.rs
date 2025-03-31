use crate::metrics::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_metric_registration() {
    let registry = MetricsRegistry::new();
    
    // Test successful registration
    assert!(registry
        .register_metric(
            "test_counter".to_string(),
            MetricType::Counter,
            "Test counter metric".to_string()
        )
        .is_ok());
    
    // Test duplicate registration
    assert!(matches!(
        registry.register_metric(
            "test_counter".to_string(),
            MetricType::Counter,
            "Duplicate metric".to_string()
        ),
        Err(MetricsError::MetricExists)
    ));
}

#[test]
fn test_counter_operations() {
    let registry = MetricsRegistry::new();
    registry
        .register_metric(
            "test_counter".to_string(),
            MetricType::Counter,
            "Test counter metric".to_string(),
        )
        .unwrap();
    
    // Test increment
    registry.increment_counter("test_counter").unwrap();
    let metric = registry.get_metric("test_counter").unwrap();
    assert_eq!(metric.get_value(), 1);
    
    // Test invalid metric type
    assert!(matches!(
        registry.set_gauge("test_counter", 5),
        Err(MetricsError::InvalidMetricType)
    ));
}

#[test]
fn test_gauge_operations() {
    let registry = MetricsRegistry::new();
    registry
        .register_metric(
            "test_gauge".to_string(),
            MetricType::Gauge,
            "Test gauge metric".to_string(),
        )
        .unwrap();
    
    // Test set value
    registry.set_gauge("test_gauge", 42).unwrap();
    let metric = registry.get_metric("test_gauge").unwrap();
    assert_eq!(metric.get_value(), 42);
    
    // Test invalid metric type
    assert!(matches!(
        registry.increment_counter("test_gauge"),
        Err(MetricsError::InvalidMetricType)
    ));
}

#[test]
fn test_histogram_operations() {
    let registry = MetricsRegistry::new();
    registry
        .register_metric(
            "test_histogram".to_string(),
            MetricType::Histogram,
            "Test histogram metric".to_string(),
        )
        .unwrap();
    
    // Test record value
    registry.record_histogram("test_histogram", 100).unwrap();
    let metric = registry.get_metric("test_histogram").unwrap();
    assert_eq!(metric.get_value(), 1);
    
    // Test invalid metric type
    assert!(matches!(
        registry.set_gauge("test_histogram", 5),
        Err(MetricsError::InvalidMetricType)
    ));
}

#[test]
fn test_metric_labels() {
    let registry = MetricsRegistry::new();
    registry
        .register_metric(
            "test_metric".to_string(),
            MetricType::Counter,
            "Test metric with labels".to_string(),
        )
        .unwrap();
    
    let metric = registry.get_metric("test_metric").unwrap();
    metric.add_label("environment".to_string(), "test".to_string());
    metric.add_label("service".to_string(), "api".to_string());
    
    let labels = metric.get_labels();
    assert_eq!(labels.get("environment").unwrap(), "test");
    assert_eq!(labels.get("service").unwrap(), "api");
}

#[test]
fn test_concurrent_operations() {
    let registry = Arc::new(MetricsRegistry::new());
    registry
        .register_metric(
            "concurrent_counter".to_string(),
            MetricType::Counter,
            "Test concurrent counter".to_string(),
        )
        .unwrap();
    
    let mut handles = vec![];
    
    // Spawn multiple threads to increment the counter
    for _ in 0..10 {
        let registry = Arc::clone(&registry);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                registry.increment_counter("concurrent_counter").unwrap();
            }
        }));
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let metric = registry.get_metric("concurrent_counter").unwrap();
    assert_eq!(metric.get_value(), 1000);
}

#[test]
fn test_metric_timestamps() {
    let registry = MetricsRegistry::new();
    registry
        .register_metric(
            "test_timestamp".to_string(),
            MetricType::Counter,
            "Test timestamp metric".to_string(),
        )
        .unwrap();
    
    let metric = registry.get_metric("test_timestamp").unwrap();
    let initial_time = metric.get_last_update();
    
    // Wait a bit to ensure time difference
    thread::sleep(Duration::from_secs(1));
    
    // Update the metric
    metric.increment();
    let new_time = metric.get_last_update();
    
    assert!(new_time > initial_time);
}

#[test]
fn test_get_all_metrics() {
    let registry = MetricsRegistry::new();
    
    // Register multiple metrics
    registry
        .register_metric(
            "counter1".to_string(),
            MetricType::Counter,
            "First counter".to_string(),
        )
        .unwrap();
    registry
        .register_metric(
            "gauge1".to_string(),
            MetricType::Gauge,
            "First gauge".to_string(),
        )
        .unwrap();
    
    // Update metrics
    registry.increment_counter("counter1").unwrap();
    registry.set_gauge("gauge1", 42).unwrap();
    
    let all_metrics = registry.get_all_metrics();
    assert_eq!(all_metrics.len(), 2);
    
    // Verify values
    let counter_value = all_metrics
        .iter()
        .find(|(name, _)| name == "counter1")
        .map(|(_, value)| value.value)
        .unwrap();
    assert_eq!(counter_value, 1);
    
    let gauge_value = all_metrics
        .iter()
        .find(|(name, _)| name == "gauge1")
        .map(|(_, value)| value.value)
        .unwrap();
    assert_eq!(gauge_value, 42);
} 