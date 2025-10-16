//! Minimal file downloader using ureq instead of heavy HTTP clients

use std::fs;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use crate::error::{GcpError, GcpResult};
use crate::github::{GitHubUrl, UrlType};
use crate::Config;

/// Minimal file downloader for public GitHub repositories
pub struct FileDownloader {
    config: Config,
    agent: ureq::Agent,
}

#[derive(serde::Deserialize)]
struct GitHubFile {
    name: String,
    path: String,
    #[serde(rename = "type")]
    file_type: String,
    download_url: Option<String>,
    content: Option<String>,
    encoding: Option<String>,
}

impl FileDownloader {
    pub fn new(config: Config) -> Self {
        let agent = ureq::AgentBuilder::new()
            .user_agent(&config.user_agent)
            .build();

        Self { config, agent }
    }

    /// Download a single file from GitHub
    pub fn download_file(&self, github_url: &GitHubUrl, destination: &str) -> GcpResult<()> {
        if github_url.url_type != UrlType::File {
            return Err(GcpError::UnsupportedOperation(
                "Not a file URL".to_string()
            ));
        }

        // Use GitHub API to get file content (base64 encoded)
        let api_url = github_url.api_url();
        let response = self.agent
            .get(&api_url)
            .call()
            .map_err(GcpError::from)?;

        if response.status() >= 400 {
            return Err(GcpError::NetworkError(format!(
                "GitHub API returned HTTP {}",
                response.status()
            )));
        }

        let file_info: GitHubFile = response.into_json()
            .map_err(GcpError::from)?;

        // Try to decode base64 content directly from API response
        if let (Some(content), Some(encoding)) = (file_info.content, file_info.encoding) {
            if encoding == "base64" {
                let clean_content = content.trim().replace('\n', "").replace('\r', "");
                let decoded = STANDARD.decode(&clean_content)
                    .map_err(GcpError::from)?;
                let data = String::from_utf8(decoded)
                    .map_err(|e| GcpError::ParseError(format!("UTF-8 decode error: {}", e)))?;

                fs::write(destination, data)?;
                return Ok(());
            }
        }

        // Fallback: try download URL if available
        if let Some(download_url) = file_info.download_url {
            let response = self.agent
                .get(&download_url)
                .call()
                .map_err(GcpError::from)?;

            if response.status() < 400 {
                let data = response.into_string()
                    .map_err(|e| GcpError::NetworkError(format!("Failed to read file: {}", e)))?;

                fs::write(destination, data)?;
                Ok(())
            } else {
                Err(GcpError::NetworkError(format!(
                    "Failed to download file: HTTP {}",
                    response.status()
                )))
            }
        } else {
            Err(GcpError::NetworkError("No file content available".to_string()))
        }
    }

    /// Download a folder from GitHub recursively
    pub fn download_folder(&self, github_url: &GitHubUrl, destination: &str) -> GcpResult<()> {
        if github_url.url_type != UrlType::Folder {
            return Err(GcpError::UnsupportedOperation(
                "Not a folder URL".to_string()
            ));
        }

        // Create destination directory
        fs::create_dir_all(destination)?;

        // Get folder contents via GitHub API
        let api_url = github_url.api_url();
        let response = self.agent
            .get(&api_url)
            .call()
            .map_err(GcpError::from)?;

        if response.status() >= 400 {
            return Err(GcpError::NetworkError(format!(
                "GitHub API returned HTTP {}",
                response.status()
            )));
        }

        let files: Vec<GitHubFile> = response.into_json()
            .map_err(GcpError::from)?;

        let mut downloaded_count = 0;

        for file in files {
            let file_path = Path::new(destination).join(&file.name);

            if file.file_type == "file" {
                if let Some(download_url) = file.download_url {
                    let response = self.agent
                        .get(&download_url)
                        .call()
                        .map_err(GcpError::from)?;

                    if response.status() < 400 {
                        let data = response.into_string()
                            .map_err(|e| GcpError::NetworkError(format!("Failed to read file: {}", e)))?;

                        fs::write(&file_path, data)?;
                        downloaded_count += 1;
                    }
                }
            } else if file.file_type == "dir" {
                // Recursively download subdirectory
                let sub_url = GitHubUrl {
                    owner: github_url.owner.clone(),
                    repo: github_url.repo.clone(),
                    path: Some(file.path.clone()),
                    ref_: github_url.ref_.clone(),
                    url_type: UrlType::Folder,
                    raw_url: String::new(),
                };

                if let Err(e) = self.download_folder(&sub_url, file_path.to_str().unwrap_or(&file.name)) {
                    eprintln!("Warning: Failed to download folder {}: {}", file.name, e);
                }
            }
        }

        eprintln!("Downloaded {} files to {}", downloaded_count, destination);
        Ok(())
    }
}