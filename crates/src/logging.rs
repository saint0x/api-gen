use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogError {
    #[error("Failed to write log")]
    WriteError,
    #[error("Invalid log level")]
    InvalidLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub fields: Vec<(String, String)>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message,
            fields: Vec::new(),
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    pub fn with_fields(mut self, fields: Vec<(String, String)>) -> Self {
        self.fields.extend(fields);
        self
    }

    pub fn format(&self) -> String {
        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S.%3f UTC");
        let fields = if self.fields.is_empty() {
            String::new()
        } else {
            format!(
                " | {}",
                self.fields
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        format!("{} | {}{} | {}", self.level, self.message, fields, timestamp)
    }
}

#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
    sequence: AtomicU64,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            sequence: AtomicU64::new(0),
        }
    }

    pub fn debug(&self, message: impl Into<String>) -> LogEntry {
        self.log(LogLevel::Debug, message)
    }

    pub fn info(&self, message: impl Into<String>) -> LogEntry {
        self.log(LogLevel::Info, message)
    }

    pub fn warn(&self, message: impl Into<String>) -> LogEntry {
        self.log(LogLevel::Warn, message)
    }

    pub fn error(&self, message: impl Into<String>) -> LogEntry {
        self.log(LogLevel::Error, message)
    }

    fn log(&self, level: LogLevel, message: impl Into<String>) -> LogEntry {
        let entry = LogEntry::new(level, message.into())
            .with_field("seq", self.next_sequence().to_string())
            .with_field("pid", std::process::id().to_string());
        
        if level >= self.level {
            println!("{}", entry.format());
        }
        
        entry
    }

    fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::Relaxed)
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
} 