use crate::request::*;
use std::net::IpAddr;
use std::str::FromStr;
use chrono::Utc;

fn create_test_headers() -> Vec<(String, String)> {
    vec![
        ("User-Agent".to_string(), "test-agent".to_string()),
        (
            "X-Request-Timestamp".to_string(),
            Utc::now().to_rfc3339(),
        ),
        ("X-Request-Id".to_string(), "test-request-id".to_string()),
    ]
}

#[test]
fn test_valid_request() {
    let validator = RequestValidator::new(
        chrono::Duration::minutes(5),
        Some(vec![IpAddr::from_str("127.0.0.1").unwrap()]),
    );

    let metadata = RequestMetadata {
        ip_address: IpAddr::from_str("127.0.0.1").unwrap(),
        user_agent: "test-agent".to_string(),
        timestamp: Utc::now(),
        request_id: "test-request-id".to_string(),
    };

    assert!(validator.validate_request(&metadata).is_ok());
}

#[test]
fn test_invalid_ip() {
    let validator = RequestValidator::new(
        chrono::Duration::minutes(5),
        Some(vec![IpAddr::from_str("127.0.0.1").unwrap()]),
    );

    let metadata = RequestMetadata {
        ip_address: IpAddr::from_str("192.168.1.1").unwrap(),
        user_agent: "test-agent".to_string(),
        timestamp: Utc::now(),
        request_id: "test-request-id".to_string(),
    };

    match validator.validate_request(&metadata) {
        Err(RequestValidationError::IpNotAllowed(_)) => (),
        _ => panic!("Expected IpNotAllowed error"),
    }
}

#[test]
fn test_old_request() {
    let validator = RequestValidator::new(
        chrono::Duration::minutes(5),
        None,
    );

    let metadata = RequestMetadata {
        ip_address: IpAddr::from_str("127.0.0.1").unwrap(),
        user_agent: "test-agent".to_string(),
        timestamp: Utc::now() - chrono::Duration::minutes(10),
        request_id: "test-request-id".to_string(),
    };

    match validator.validate_request(&metadata) {
        Err(RequestValidationError::RequestTooOld(_)) => (),
        _ => panic!("Expected RequestTooOld error"),
    }
}

#[test]
fn test_extract_metadata() {
    let headers = create_test_headers();
    let ip = IpAddr::from_str("127.0.0.1").unwrap();

    let metadata = RequestValidator::extract_metadata(&headers, ip).unwrap();
    assert_eq!(metadata.ip_address, ip);
    assert_eq!(metadata.user_agent, "test-agent");
    assert_eq!(metadata.request_id, "test-request-id");
}

#[test]
fn test_missing_headers() {
    let headers = vec![("User-Agent".to_string(), "test-agent".to_string())];
    let ip = IpAddr::from_str("127.0.0.1").unwrap();

    match RequestValidator::extract_metadata(&headers, ip) {
        Err(RequestValidationError::MissingHeader(_)) => (),
        _ => panic!("Expected MissingHeader error"),
    }
} 