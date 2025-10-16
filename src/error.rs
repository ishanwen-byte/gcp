//! Minimal error handling for lightweight GitHub downloader

use std::string::String;

/// Minimal error type without thiserror dependency
#[derive(Debug, Clone)]
pub enum GcpError {
    InvalidUrl(String),
    NetworkError(String),
    FileSystemError(String),
    ParseError(String),
    UnsupportedOperation(String),
    IoError(String),
    NotFound(String),
    PermissionDenied(String),
}

impl core::fmt::Display for GcpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GcpError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            GcpError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            GcpError::FileSystemError(msg) => write!(f, "Filesystem error: {}", msg),
            GcpError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            GcpError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            GcpError::IoError(msg) => write!(f, "IO error: {}", msg),
            GcpError::NotFound(msg) => write!(f, "Not found: {}", msg),
            GcpError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

/// Result type alias
pub type GcpResult<T> = Result<T, GcpError>;

// Implement standard error traits
impl std::error::Error for GcpError {}

// Conversion from URL parse errors
impl From<url::ParseError> for GcpError {
    fn from(err: url::ParseError) -> Self {
        GcpError::InvalidUrl(err.to_string())
    }
}

// Conversion from serde_json errors
impl From<serde_json::Error> for GcpError {
    fn from(err: serde_json::Error) -> Self {
        GcpError::ParseError(err.to_string())
    }
}

// Conversion from base64 errors
impl From<base64::DecodeError> for GcpError {
    fn from(err: base64::DecodeError) -> Self {
        GcpError::ParseError(format!("Base64 decode error: {}", err))
    }
}

// Conversion from ureq errors
impl From<ureq::Error> for GcpError {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::Transport(transport_err) => {
                GcpError::NetworkError(format!("Transport error: {}", transport_err))
            }
            ureq::Error::Status(status, response) => {
                GcpError::NetworkError(format!("HTTP {}: {}", status, response.status_text()))
            }
        }
    }
}

// Conversion from std::io errors
impl From<std::io::Error> for GcpError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => GcpError::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => GcpError::PermissionDenied(err.to_string()),
            _ => GcpError::IoError(err.to_string()),
        }
    }
}