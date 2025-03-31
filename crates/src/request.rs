use std::net::IpAddr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq)]
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