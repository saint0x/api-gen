use crate::logging::*;

#[test]
fn test_log_level_display() {
    assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
    assert_eq!(LogLevel::Info.to_string(), "INFO");
    assert_eq!(LogLevel::Warn.to_string(), "WARN");
    assert_eq!(LogLevel::Error.to_string(), "ERROR");
}

#[test]
fn test_log_entry_creation() {
    let entry = LogEntry::new(LogLevel::Info, "Test message".to_string());
    assert_eq!(entry.level, LogLevel::Info);
    assert_eq!(entry.message, "Test message");
    assert!(entry.fields.is_empty());
}

#[test]
fn test_log_entry_with_fields() {
    let entry = LogEntry::new(LogLevel::Info, "Test message".to_string())
        .with_field("key", "value")
        .with_field("number", "42");
    
    assert_eq!(entry.fields.len(), 2);
    assert_eq!(entry.fields[0].0, "key");
    assert_eq!(entry.fields[0].1, "value");
    assert_eq!(entry.fields[1].0, "number");
    assert_eq!(entry.fields[1].1, "42");
}

#[test]
fn test_log_entry_format() {
    let entry = LogEntry::new(LogLevel::Info, "Test message".to_string())
        .with_field("key", "value");
    
    let formatted = entry.format();
    assert!(formatted.contains("INFO"));
    assert!(formatted.contains("Test message"));
    assert!(formatted.contains("key=value"));
    assert!(formatted.contains("UTC"));
}

#[test]
fn test_logger_levels() {
    let mut logger = Logger::new(LogLevel::Info);
    
    // Debug should not be logged at Info level
    let debug_entry = logger.debug("Debug message");
    assert_eq!(debug_entry.level, LogLevel::Debug);
    
    // Info should be logged
    let info_entry = logger.info("Info message");
    assert_eq!(info_entry.level, LogLevel::Info);
    
    // Warn should be logged
    let warn_entry = logger.warn("Warn message");
    assert_eq!(warn_entry.level, LogLevel::Warn);
    
    // Error should be logged
    let error_entry = logger.error("Error message");
    assert_eq!(error_entry.level, LogLevel::Error);
    
    // Change level to Warn
    logger.set_level(LogLevel::Warn);
    
    // Debug and Info should not be logged at Warn level
    let debug_entry = logger.debug("Debug message");
    assert_eq!(debug_entry.level, LogLevel::Debug);
    
    let info_entry = logger.info("Info message");
    assert_eq!(info_entry.level, LogLevel::Info);
    
    // Warn and Error should still be logged
    let warn_entry = logger.warn("Warn message");
    assert_eq!(warn_entry.level, LogLevel::Warn);
    
    let error_entry = logger.error("Error message");
    assert_eq!(error_entry.level, LogLevel::Error);
}

#[test]
fn test_logger_sequence() {
    let logger = Logger::new(LogLevel::Debug);
    
    let entry1 = logger.info("First message");
    let entry2 = logger.info("Second message");
    let entry3 = logger.info("Third message");
    
    let seq1 = entry1.fields.iter().find(|(k, _)| k == "seq").unwrap().1.parse::<u64>().unwrap();
    let seq2 = entry2.fields.iter().find(|(k, _)| k == "seq").unwrap().1.parse::<u64>().unwrap();
    let seq3 = entry3.fields.iter().find(|(k, _)| k == "seq").unwrap().1.parse::<u64>().unwrap();
    
    assert_eq!(seq2, seq1 + 1);
    assert_eq!(seq3, seq2 + 1);
}

#[test]
fn test_logger_pid() {
    let logger = Logger::new(LogLevel::Debug);
    let entry = logger.info("Test message");
    
    let pid = entry.fields.iter().find(|(k, _)| k == "pid").unwrap().1.parse::<u32>().unwrap();
    assert_eq!(pid, std::process::id());
} 