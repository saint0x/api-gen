use std::net::IpAddr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestValidationError {
    #[error("Invalid request signature")]
    InvalidSignature,
    #[error("IP address not allowed: {0}")]
    IpNotAllowed(IpAddr),
    #[error("Request timestamp too old: {0}")]
    RequestTooOld(DateTime<Utc>),
    #[error("Missing required header: {0}")]
    MissingHeader(String),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

#[derive(Debug, Clone)]
pub struct RequestValidator {
    max_request_age: chrono::Duration,
    allowed_ips: Option<Vec<IpAddr>>,
}

impl RequestValidator {
    pub fn new(max_request_age: chrono::Duration, allowed_ips: Option<Vec<IpAddr>>) -> Self {
        Self {
            max_request_age,
            allowed_ips,
        }
    }

    pub fn validate_request(&self, metadata: &RequestMetadata) -> Result<(), RequestValidationError> {
        // Validate request age
        let age = Utc::now() - metadata.timestamp;
        if age > self.max_request_age {
            return Err(RequestValidationError::RequestTooOld(metadata.timestamp));
        }

        // Validate IP if allowed_ips is set
        if let Some(allowed_ips) = &self.allowed_ips {
            if !allowed_ips.contains(&metadata.ip_address) {
                return Err(RequestValidationError::IpNotAllowed(metadata.ip_address));
            }
        }

        Ok(())
    }

    pub fn validate_signature(
        &self,
        _request_body: &[u8],
        _signature: &str,
        _timestamp: &str,
        _api_key: &str,
    ) -> Result<(), RequestValidationError> {
        // TODO: Implement HMAC signature validation
        // This is a placeholder for the actual signature validation logic
        // In production, you would:
        // 1. Concatenate request_body + timestamp
        // 2. Generate HMAC using api_key as secret
        // 3. Compare with provided signature
        Ok(())
    }

    pub fn extract_metadata(
        headers: &[(String, String)],
        ip: IpAddr,
    ) -> Result<RequestMetadata, RequestValidationError> {
        let user_agent = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("user-agent"))
            .map(|(_, v)| v.clone())
            .ok_or_else(|| RequestValidationError::MissingHeader("User-Agent".to_string()))?;

        let timestamp = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("x-request-timestamp"))
            .map(|(_, v)| v.clone())
            .ok_or_else(|| RequestValidationError::MissingHeader("X-Request-Timestamp".to_string()))?;

        let request_id = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("x-request-id"))
            .map(|(_, v)| v.clone())
            .ok_or_else(|| RequestValidationError::MissingHeader("X-Request-Id".to_string()))?;

        let timestamp = DateTime::parse_from_rfc3339(&timestamp)
            .map_err(|_| RequestValidationError::InvalidHeaderValue("Invalid timestamp format".to_string()))?
            .with_timezone(&Utc);

        Ok(RequestMetadata {
            ip_address: ip,
            user_agent,
            timestamp,
            request_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

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
} 