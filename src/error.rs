use std::path::PathBuf;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GcpError {
    #[error("Invalid GitHub URL: {url}")]
    InvalidUrl { url: String },

    #[error("GitHub API error: {message} (status: {status})")]
    GitHubApi { status: u16, message: String },

    #[error("Rate limit exceeded. Resets at: {reset_time}")]
    RateLimit { reset_time: DateTime<Utc> },

    #[error("Authentication failed: {reason}")]
    Authentication { reason: String },

    #[error("File system error: {path} - {reason}")]
    FileSystem { path: PathBuf, reason: String },

    #[error("Network error: {source}")]
    Network { #[from] source: reqwest::Error },

    #[error("Download failed: {file} - {reason}")]
    DownloadFailed { file: String, reason: String },

    #[error("File conflict at {path}: {existing} vs {incoming}")]
    FileConflict { path: PathBuf, existing: String, incoming: String },

    #[error("IO error: {source}")]
    Io { #[from] source: std::io::Error },

    #[error("URL parsing error: {source}")]
    UrlParse { #[from] source: url::ParseError },

    #[error("JSON error: {source}")]
    Json { #[from] source: serde_json::Error },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Invalid file path: {path}")]
    InvalidPath { path: String },

    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    FileTooLarge { size: u64, limit: u64 },

    #[error("Operation cancelled")]
    Cancelled,

    #[error("File IO error: {path} - {source}")]
    FileIo { path: PathBuf, #[source] source: std::io::Error },

    #[error("Invalid operation: {operation} - {reason}")]
    InvalidOperation { operation: String, reason: String },
}

pub type Result<T> = std::result::Result<T, GcpError>;

impl GcpError {
    pub fn is_retryable(&self) -> bool {
        match self {
            GcpError::Network { .. } => true,
            GcpError::RateLimit { .. } => true,
            GcpError::GitHubApi { status, .. } => {
                // Retry on server errors (5xx) and rate limiting (429)
                *status >= 500 || *status == 429
            }
            _ => false,
        }
    }

    pub fn is_auth_error(&self) -> bool {
        match self {
            GcpError::Authentication { .. } => true,
            GcpError::GitHubApi { status, .. } => *status == 401 || *status == 403,
            _ => false,
        }
    }

    pub fn is_not_found(&self) -> bool {
        match self {
            GcpError::GitHubApi { status, .. } => *status == 404,
            _ => false,
        }
    }
}