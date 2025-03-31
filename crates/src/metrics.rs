use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use dashmap::DashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Metric not found")]
    MetricNotFound,
    #[error("Invalid metric type")]
    InvalidMetricType,
    #[error("Metric already exists")]
    MetricExists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone)]
pub struct MetricValue {
    pub value: u64,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    value: AtomicU64,
    last_update: AtomicU64,
    labels: DashMap<String, String>,
}

impl Metric {
    pub fn new(name: String, metric_type: MetricType, description: String) -> Self {
        Self {
            name,
            metric_type,
            description,
            value: AtomicU64::new(0),
            last_update: AtomicU64::new(Utc::now().timestamp() as u64),
            labels: DashMap::new(),
        }
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
        self.last_update.store(Utc::now().timestamp() as u64, Ordering::Relaxed);
    }

    pub fn decrement(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
        self.last_update.store(Utc::now().timestamp() as u64, Ordering::Relaxed);
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
        self.last_update.store(Utc::now().timestamp() as u64, Ordering::Relaxed);
    }

    pub fn get_value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn get_last_update(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.last_update.load(Ordering::Relaxed) as i64, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    pub fn add_label(&self, key: String, value: String) {
        self.labels.insert(key, value);
    }

    pub fn get_labels(&self) -> HashMap<String, String> {
        self.labels.iter().map(|r| (r.key().clone(), r.value().clone())).collect()
    }
}

#[derive(Debug)]
pub struct MetricsRegistry {
    metrics: DashMap<String, Arc<Metric>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            metrics: DashMap::new(),
        }
    }

    pub fn register_metric(
        &self,
        name: String,
        metric_type: MetricType,
        description: String,
    ) -> Result<(), MetricsError> {
        if self.metrics.contains_key(&name) {
            return Err(MetricsError::MetricExists);
        }

        let metric = Arc::new(Metric::new(name.clone(), metric_type, description));
        self.metrics.insert(name, metric);
        Ok(())
    }

    pub fn get_metric(&self, name: &str) -> Result<Arc<Metric>, MetricsError> {
        self.metrics
            .get(name)
            .map(|r| r.value().clone())
            .ok_or(MetricsError::MetricNotFound)
    }

    pub fn increment_counter(&self, name: &str) -> Result<(), MetricsError> {
        let metric = self.get_metric(name)?;
        if metric.metric_type != MetricType::Counter {
            return Err(MetricsError::InvalidMetricType);
        }
        metric.increment();
        Ok(())
    }

    pub fn set_gauge(&self, name: &str, value: u64) -> Result<(), MetricsError> {
        let metric = self.get_metric(name)?;
        if metric.metric_type != MetricType::Gauge {
            return Err(MetricsError::InvalidMetricType);
        }
        metric.set(value);
        Ok(())
    }

    pub fn record_histogram(&self, name: &str, _value: u64) -> Result<(), MetricsError> {
        let metric = self.get_metric(name)?;
        if metric.metric_type != MetricType::Histogram {
            return Err(MetricsError::InvalidMetricType);
        }
        metric.increment(); // For histogram, we just count occurrences
        Ok(())
    }

    pub fn get_all_metrics(&self) -> Vec<(String, MetricValue)> {
        self.metrics
            .iter()
            .map(|r| {
                let metric = r.value();
                (
                    metric.name.clone(),
                    MetricValue {
                        value: metric.get_value(),
                        timestamp: metric.get_last_update(),
                        labels: metric.get_labels(),
                    },
                )
            })
            .collect()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
} 