use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn, error};

use crate::error::{GcpError, Result};
use crate::github::{GitHubClient, GitHubUrl, GitHubFile};
use crate::filesystem::{create_intermediate_dirs, ensure_destination_dir};
use crate::downloader::{FileDownloader, ProgressReporter};

/// Downloads entire folders from GitHub repositories
pub struct FolderDownloader {
    github_client: Arc<GitHubClient>,
    file_downloader: Arc<FileDownloader>,
    progress: Option<Arc<ProgressReporter>>,
}

impl FolderDownloader {
    pub fn new(github_client: Arc<GitHubClient>) -> Self {
        let file_downloader = Arc::new(FileDownloader::new(github_client.clone()));
        Self {
            github_client,
            file_downloader,
            progress: None,
        }
    }

    pub fn with_progress(mut self, progress: Arc<ProgressReporter>) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Download an entire folder from GitHub recursively
    pub async fn download_folder(&self, github_url: &GitHubUrl, destination: &PathBuf, force: bool) -> Result<usize> {
        debug!("Downloading folder from {} to {:?}", github_url.api_path(), destination);

        // Ensure the URL type is correct
        if github_url.url_type != crate::github::UrlType::Folder {
            return Err(GcpError::InvalidOperation {
                operation: "download_folder".to_string(),
                reason: format!("URL type is not a folder: {:?}", github_url.url_type),
            });
        }

        // Ensure destination directory exists
        ensure_destination_dir(destination)?;
        create_intermediate_dirs(destination)?;

        // Start folder download
        let mut downloaded_files = 0;
        self.download_folder_recursive(github_url, destination, &mut downloaded_files, force).await?;

        info!("Successfully downloaded {} files to {}", downloaded_files, destination.display());
        Ok(downloaded_files)
    }

    /// Recursively download folder contents
    fn download_folder_recursive<'a>(
        &'a self,
        github_url: &'a GitHubUrl,
        destination: &'a PathBuf,
        downloaded_files: &'a mut usize,
        force: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            debug!("Processing folder: {}", github_url.api_path());

            // Get folder contents from GitHub API
            let contents = self.get_folder_contents(github_url).await?;

            for item in contents {
                let item_destination = destination.join(&item.name);

                if item.is_file() {
                    // Download file
                    debug!("Downloading file: {}", item.path);

                    let file_url = GitHubUrl {
                        owner: github_url.owner.clone(),
                        repo: github_url.repo.clone(),
                        path: Some(item.path.clone()),
                        ref_: github_url.ref_.clone(),
                        url_type: crate::github::UrlType::File,
                    };

                    match self.file_downloader.download_file(&file_url, &item_destination, force).await {
                        Ok(_) => {
                            *downloaded_files += 1;
                            if let Some(ref progress) = self.progress {
                                progress.set_message(&format!("Downloaded {} files", downloaded_files));
                            }
                        }
                        Err(e) => {
                            warn!("Failed to download file {}: {}", item.path, e);
                            // Continue with other files even if one fails
                        }
                    }
                } else if item.is_directory() {
                    // Recursively download subfolder
                    debug!("Entering subdirectory: {}", item.path);

                    let folder_url = GitHubUrl {
                        owner: github_url.owner.clone(),
                        repo: github_url.repo.clone(),
                        path: Some(item.path.clone()),
                        ref_: github_url.ref_.clone(),
                        url_type: crate::github::UrlType::Folder,
                    };

                    // Create subdirectory
                    create_intermediate_dirs(&item_destination)?;

                    self.download_folder_recursive(&folder_url, &item_destination, downloaded_files, force).await?;
                } else if item.is_submodule() {
                    debug!("Skipping submodule: {}", item.path);
                    // TODO: Handle submodules if needed
                } else if item.is_symlink() {
                    debug!("Skipping symlink: {}", item.path);
                    // TODO: Handle symlinks if needed
                }
            }

            Ok(())
        })
    }

    /// Get folder contents from GitHub API
    async fn get_folder_contents(&self, github_url: &GitHubUrl) -> Result<Vec<GitHubFile>> {
        debug!("Getting folder contents for: {}/{}@{}",
               github_url.owner,
               github_url.repo,
               github_url.ref_.as_deref().unwrap_or("main"));

        let path = github_url.path.as_deref().unwrap_or("");
        let ref_ = github_url.ref_.as_deref().unwrap_or("main");

        // Use GitHub API to list directory contents
        let handler = self.github_client.client.repos(&github_url.owner, &github_url.repo);

        // Try to get content using the GitHub Contents API
        match handler.get_content().path(path).r#ref(ref_).send().await {
            Ok(contents) => {
                debug!("Successfully fetched folder contents, found {} items", contents.items.len());

                // Convert octocrab Content items to our GitHubFile format
                let mut github_files = Vec::new();

                for item in contents.items {
                    let github_file = GitHubFile {
                        name: item.name,
                        path: item.path,
                        sha: item.sha,
                        size: item.size as i64,
                        url: item.url,
                        html_url: item.html_url.unwrap_or_else(|| "".to_string()),
                        git_url: item.git_url.unwrap_or_else(|| "".to_string()),
                        download_url: item.download_url,
                        file_type: item.r#type,
                        content: item.content,
                        encoding: item.encoding,
                    };
                    github_files.push(github_file);
                }

                Ok(github_files)
            }
            Err(e) => {
                error!("Failed to get folder contents from GitHub API: {}", e);

                // Fallback: Try to construct from raw URL if possible
                if let Some(raw_url) = github_url.raw_url() {
                    warn!("GitHub API failed, attempting fallback approach for folder");
                    self.get_folder_contents_fallback(github_url).await
                } else {
                    Err(GcpError::GitHubApi {
                        status: 0,
                        message: format!("Failed to get folder contents: {}", e),
                    })
                }
            }
        }
    }

    /// Fallback method to get folder contents when API fails
    async fn get_folder_contents_fallback(&self, _github_url: &GitHubUrl) -> Result<Vec<GitHubFile>> {
        debug!("Using fallback method for folder contents");

        // For now, return an empty list but log the attempt
        // In a more sophisticated implementation, we could try to:
        // 1. Use the GitHub Search API to find files
        // 2. Parse HTML from the GitHub web interface
        // 3. Use a combination of known file patterns

        warn!("Fallback method not implemented yet, returning empty folder contents");
        Ok(vec![])
    }

    /// Validate that the folder can be downloaded
    pub async fn validate_download(&self, github_url: &GitHubUrl) -> Result<bool> {
        if github_url.url_type != crate::github::UrlType::Folder {
            return Ok(false);
        }

        // Try to get folder contents to validate accessibility
        match self.get_folder_contents(github_url).await {
            Ok(contents) => {
                debug!("Folder validation successful, found {} items", contents.len());
                Ok(true)
            }
            Err(e) => {
                error!("Folder validation failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Estimate total files in folder for progress reporting
    fn estimate_file_count<'a>(
        &'a self,
        github_url: &'a GitHubUrl,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<usize>> + Send + 'a>> {
        Box::pin(async move {
            match self.get_folder_contents(github_url).await {
                Ok(contents) => {
                    let mut count = 0;
                    for item in contents {
                        if item.is_file() {
                            count += 1;
                        } else if item.is_directory() {
                            // Recursively count files in subdirectories
                            let folder_url = GitHubUrl {
                                owner: github_url.owner.clone(),
                                repo: github_url.repo.clone(),
                                path: Some(item.path.clone()),
                                ref_: github_url.ref_.clone(),
                                url_type: crate::github::UrlType::Folder,
                            };
                            count += self.estimate_file_count(&folder_url).await?;
                        }
                    }
                    Ok(count)
                }
                Err(_) => Ok(0),
            }
        })
    }
}