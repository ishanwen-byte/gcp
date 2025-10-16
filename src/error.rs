//! Minimal error handling for lightweight GitHub downloader

/// Minimal error type without thiserror dependency
#[derive(Debug, Clone)]
pub enum GcpError {
    InvalidUrl(String),
    NetworkError(String),
    ParseError(String),
    UnsupportedOperation(String),
    NotFound(String),
    PermissionDenied(String),
}

impl core::fmt::Display for GcpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GcpError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            GcpError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            GcpError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            GcpError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            GcpError::NotFound(msg) => write!(f, "Not found: {}", msg),
            GcpError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

/// Result type alias
pub type GcpResult<T> = Result<T, GcpError>;

// Implement standard error traits
impl std::error::Error for GcpError {}



// Conversion from base64 errors
impl From<base64::DecodeError> for GcpError {
    fn from(err: base64::DecodeError) -> Self {
        GcpError::ParseError(format!("Base64 decode error: {}", err))
    }
}

// Conversion from attohttpc errors
impl From<attohttpc::Error> for GcpError {
    fn from(err: attohttpc::Error) -> Self {
        GcpError::NetworkError(format!("HTTP error: {}", err))
    }
}

// Conversion from std::io errors
impl From<std::io::Error> for GcpError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => GcpError::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => GcpError::PermissionDenied(err.to_string()),
            _ => GcpError::NetworkError(err.to_string()),
        }
    }
}