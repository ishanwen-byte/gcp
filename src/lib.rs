//! Minimal GitHub Copier - lightweight version
//! Downloads files/folders from public GitHub repositories

pub mod error;
pub mod github;
pub mod downloader;
pub use error::{GcpError, GcpResult};
pub use github::{GitHubUrl, UrlType};
pub use downloader::FileDownloader;

/// Main entry point for downloading from GitHub
pub fn download_from_github(
    url_str: &str,
    destination: &str,
) -> GcpResult<()> {
    // Parse the GitHub URL
    let github_url = GitHubUrl::parse(url_str)?;

    // Create downloader
    let downloader = FileDownloader::new();

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
    match github_url.url_type {
        UrlType::File => {
            downloader.download_file(&github_url, &final_destination)
        }
        UrlType::Folder => {
            downloader.download_folder(&github_url, &final_destination)
        }
        UrlType::Repository => {
            Err(GcpError::UnsupportedOperation(
                "Repository downloads not supported in minimal version".to_string()
            ))
        }
    }
}