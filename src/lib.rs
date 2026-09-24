//! Minimal GitHub Copier - lightweight version
//! Downloads files/folders from public GitHub repositories

// Core functionality
pub mod base64;
pub mod client;
pub mod error;
pub mod github;
pub mod json;

// Public API exports
pub use client::GitHubClient;
pub use error::{GcpError, GcpResult};
pub use github::{GitHubUrl, UrlType};

/// Main entry point for downloading from GitHub
pub fn download_from_github(url_str: &str, destination: &str) -> GcpResult<()> {
    // Parse the GitHub URL
    let github_url = GitHubUrl::parse(url_str)?;

    // Create client
    let client = GitHubClient::new()?;

    // If destination is "." (default), try to use original filename
    let final_destination = if destination == "." {
        match github_url.filename() {
            Some(filename) => filename,
            None => {
                return Err(GcpError::InvalidUrl(
                    "Cannot extract filename from URL and no destination provided".to_string(),
                ));
            }
        }
    } else {
        destination.to_string()
    };

    // A trailing path separator on a file destination makes it a directory
    // reference on Windows ("out.txt\" is invalid). Strip it so the intent
    // (write file `out.txt`) works across platforms. Keep the original when
    // trimming would produce an empty path (e.g. destination "/").
    let final_destination = match final_destination.trim_end_matches(['/', '\\']) {
        "" => final_destination,
        trimmed => trimmed.to_string(),
    };

    // Download based on URL type
    client.download(&github_url, &final_destination)
}
