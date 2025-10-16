use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, error};

use crate::error::{GcpError, Result};
use crate::github::{GitHubClient, GitHubUrl};
use crate::filesystem::{create_intermediate_dirs, resolve_conflict, ensure_destination_dir};
use crate::downloader::ProgressReporter;

/// Downloads individual files from GitHub repositories
pub struct FileDownloader {
    github_client: Arc<GitHubClient>,
    progress: Option<Arc<ProgressReporter>>,
}

impl FileDownloader {
    pub fn new(github_client: Arc<GitHubClient>) -> Self {
        Self {
            github_client,
            progress: None,
        }
    }

    pub fn with_progress(mut self, progress: Arc<ProgressReporter>) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Download a single file from GitHub
    pub async fn download_file(&self, github_url: &GitHubUrl, destination: &PathBuf, force: bool) -> Result<PathBuf> {
        debug!("Downloading file from {} to {:?}", github_url.raw_url().unwrap_or_default(), destination);

        // Ensure the file type is correct
        if github_url.url_type != crate::github::UrlType::File {
            return Err(GcpError::InvalidOperation {
                operation: "download_file".to_string(),
                reason: format!("URL type is not a file: {:?}", github_url.url_type),
            });
        }

        // Ensure destination directory exists
        ensure_destination_dir(destination)?;

        // Handle file conflicts
        let final_destination = if force && destination.exists() {
            // Force overwrite
            destination.clone()
        } else {
            // Auto-rename to avoid conflicts
            resolve_conflict(destination)
        };
        create_intermediate_dirs(&final_destination)?;

        // Try to use raw URL first (easier, no auth required for public repos)
        if let Some(raw_url) = github_url.raw_url() {
            debug!("Attempting download from raw URL: {}", raw_url);
            match self.download_from_raw_url(&raw_url, &final_destination).await {
                Ok(path) => return Ok(path),
                Err(e) => {
                    debug!("Raw URL download failed, falling back to GitHub API: {}", e);
                }
            }
        }

        // Fallback to GitHub API
        debug!("Using GitHub API for file download");
        let (content, size) = self.github_client
            .get_file_info(
                &github_url.owner,
                &github_url.repo,
                github_url.path.as_deref().unwrap_or(""),
                github_url.ref_.as_deref()
            )
            .await?;

        // Write content to file
        tokio::fs::write(&final_destination, content).await
            .map_err(|e| GcpError::FileIo {
                path: final_destination.clone(),
                source: e,
            })?;

  
        Ok(final_destination)
    }

    /// Download file from raw URL (fallback method)
    pub async fn download_from_raw_url(&self, raw_url: &str, destination: &PathBuf) -> Result<PathBuf> {
        debug!("Downloading from raw URL: {}", raw_url);

        // Ensure destination directory exists
        ensure_destination_dir(destination)?;

        // Resolve conflicts if destination exists
        let final_destination = resolve_conflict(destination);
        create_intermediate_dirs(&final_destination)?;

        // Download content using HTTP client
        let content = self.github_client.download_file_content(raw_url).await?;

        // Write content to file
        tokio::fs::write(&final_destination, content).await
            .map_err(|e| GcpError::FileIo {
                path: final_destination.clone(),
                source: e,
            })?;

    
        Ok(final_destination)
    }

    /// Validate that the file can be downloaded
    pub async fn validate_download(&self, github_url: &GitHubUrl) -> Result<bool> {
        if github_url.url_type != crate::github::UrlType::File {
            return Ok(false);
        }

        // Try to get file info to validate accessibility
        match self.github_client
            .get_file_info(
                &github_url.owner,
                &github_url.repo,
                github_url.path.as_deref().unwrap_or(""),
                github_url.ref_.as_deref()
            )
            .await
        {
            Ok((_, size)) => {
                debug!("File validation successful, size: {} bytes", size);
                Ok(true)
            }
            Err(e) => {
                error!("File validation failed: {}", e);
                Ok(false)
            }
        }
    }
}