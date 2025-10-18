//! Minimal GitHub Copier - lightweight version
//! Downloads files/folders from public GitHub repositories

// Core functionality
pub mod error;
pub mod github;
pub mod client;
pub mod base64;

// Public API exports
pub use error::{GcpError, GcpResult};
pub use github::{GitHubUrl, UrlType};
pub use client::GitHubClient;

/// Main entry point for downloading from GitHub
pub fn download_from_github(
    url_str: &str,
    destination: &str,
) -> GcpResult<()> {
    // Parse the GitHub URL
    let github_url = GitHubUrl::parse(url_str)?;

    // Create client
    let client = GitHubClient::new()?;

    // If destination is "." (default), try to use original filename
    let final_destination = if destination == "." {
        match github_url.filename() {
            Some(filename) => filename,
            None => return Err(GcpError::InvalidUrl(
                "Cannot extract filename from URL and no destination provided".to_string()
            ))
        }
    } else {
        destination.to_string()
    };

    // Download based on URL type
    client.download(&github_url, &final_destination)
}