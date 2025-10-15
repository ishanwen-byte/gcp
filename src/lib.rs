pub mod error;
pub mod github;
pub mod downloader;
pub mod filesystem;

pub use error::{GcpError, Result};

use tracing::info;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub github: GitHubConfig,
    pub download: DownloadConfig,
    pub filesystem: FilesystemConfig,
}

#[derive(Debug, Clone)]
pub struct GitHubConfig {
    pub api_url: String,
    pub max_concurrent_requests: usize,
    pub retry_attempts: u32,
    pub rate_limit_buffer: usize,
}

#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub chunk_size: usize,
    pub max_file_size: u64,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct FilesystemConfig {
    pub default_permissions: Option<u32>,
    pub preserve_timestamps: bool,
    pub create_intermediate_dirs: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github: GitHubConfig::default(),
            download: DownloadConfig::default(),
            filesystem: FilesystemConfig::default(),
        }
    }
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.github.com".to_string(),
            max_concurrent_requests: 10,
            retry_attempts: 3,
            rate_limit_buffer: 5, // Keep 5 requests as buffer
        }
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024 * 1024, // 1MB chunks
            max_file_size: 100 * 1024 * 1024, // 100MB max file size
            timeout_seconds: 30,
        }
    }
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            default_permissions: Some(0o644),
            preserve_timestamps: false,
            create_intermediate_dirs: true,
        }
    }
}

pub fn init_logging(verbose: bool, quiet: bool) {
    use tracing_subscriber::{fmt, EnvFilter};

    let level = if verbose {
        "debug"
    } else if quiet {
        "error"
    } else {
        "info"
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    info!("GitHub Copy Tool initialized");
}

pub fn get_cache_dir() -> Result<PathBuf> {
    match dirs::cache_dir() {
        Some(mut path) => {
            path.push("gcp");
            std::fs::create_dir_all(&path)?;
            Ok(path)
        }
        None => Err(GcpError::Config {
            message: "Could not determine cache directory".to_string(),
        }),
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    match dirs::config_dir() {
        Some(mut path) => {
            path.push("gcp");
            std::fs::create_dir_all(&path)?;
            Ok(path)
        }
        None => Err(GcpError::Config {
            message: "Could not determine config directory".to_string(),
        }),
    }
}