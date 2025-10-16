//! Minimal GitHub Copier - lightweight version
//! Downloads files/folders from public GitHub repositories

use std::string::String;

pub mod error;
pub mod github;
pub mod downloader;

pub use error::{GcpError, GcpResult};
pub use github::{GitHubUrl, UrlType};
pub use downloader::FileDownloader;

/// Simple configuration for the minimal downloader
#[derive(Debug, Clone)]
pub struct Config {
    pub user_agent: String,
    pub timeout_secs: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_agent: "gcp-minimal/0.1.0".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Main entry point for downloading from GitHub
pub fn download_from_github(
    url_str: &str,
    destination: &str,
    config: Option<Config>,
) -> GcpResult<()> {
    let config = config.unwrap_or_default();

    // Parse the GitHub URL
    let github_url = GitHubUrl::parse(url_str)?;

    // Create downloader
    let downloader = FileDownloader::new(config);

    // Download based on URL type
    match github_url.url_type {
        UrlType::File => {
            downloader.download_file(&github_url, destination)
        }
        UrlType::Folder => {
            downloader.download_folder(&github_url, destination)
        }
        UrlType::Repository => {
            Err(GcpError::UnsupportedOperation(
                "Repository downloads not supported in minimal version".to_string()
            ))
        }
    }
}