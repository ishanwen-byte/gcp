//! Minimal file downloader using ureq instead of heavy HTTP clients

use std::fs;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose::STANDARD};
// use ureq::AgentBuilder;
use crate::error::{GcpError, GcpResult};
use crate::github::{GitHubUrl, UrlType};

/// Minimal file downloader for public GitHub repositories
pub struct FileDownloader {
    client: attohttpc::Session,
}

struct GitHubFile {
    name: String,
    path: String,
    file_type: String,  // 可以是 "file", "dir"
    download_url: Option<String>,
    content: Option<String>,
    encoding: Option<String>,  // 通常是 "base64"
}

impl GitHubFile {
    /// Minimal JSON parser for GitHub API response
    fn from_json(json_str: &str) -> Result<Self, GcpError> {
        let mut file = GitHubFile {
            name: String::new(),
            path: String::new(),
            file_type: String::new(),
            download_url: None,
            content: None,
            encoding: None,
        };

        // Simple field extraction
        file.name = extract_json_field(json_str, "name");
        file.path = extract_json_field(json_str, "path");
        file.file_type = extract_json_field(json_str, "type");
        file.download_url = extract_optional_json_field(json_str, "download_url");
        file.content = extract_optional_json_field(json_str, "content");
        file.encoding = extract_optional_json_field(json_str, "encoding");

        if file.name.is_empty() {
            return Err(GcpError::ParseError("Invalid JSON format".to_string()));
        }

        Ok(file)
    }
}

/// Extract field value from JSON string
fn extract_json_field(json_str: &str, field: &str) -> String {
    let pattern = format!("\"{}\":", field);
    if let Some(start) = json_str.find(&pattern) {
        let after_field = &json_str[start + pattern.len()..];
        // Find first quote after colon
        if let Some(quote_start) = after_field.find('"') {
            let after_first_quote = &after_field[quote_start + 1..];
            // Find second quote
            if let Some(quote_end) = after_first_quote.find('"') {
                return after_first_quote[..quote_end].to_string();
            }
        }
    }
    String::new()
}

/// Extract optional field value from JSON string
fn extract_optional_json_field(json_str: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\":", field);
    if let Some(start) = json_str.find(&pattern) {
        let after_field = &json_str[start + pattern.len()..];
        if after_field.trim_start().starts_with("null") {
            return None;
        }
        // Find first quote after colon
        if let Some(quote_start) = after_field.find('"') {
            let after_first_quote = &after_field[quote_start + 1..];
            // Find second quote
            if let Some(quote_end) = after_first_quote.find('"') {
                let value = after_first_quote[..quote_end].to_string();
                return if value.is_empty() {
                    None
                } else {
                    Some(value)
                };
            }
        }
    }
    None
}


/// Parse GitHub API file array response
fn parse_github_file_array(json_str: &str) -> Result<Vec<GitHubFile>, GcpError> {
    let mut files = Vec::new();

    // Find array start
    let json_str = json_str.trim();
    if !json_str.starts_with('[') {
        return Err(GcpError::ParseError("Expected JSON array".to_string()));
    }

    // Remove outer brackets
    let array_content = &json_str[1..json_str.len().saturating_sub(1)];

    // Split by object boundaries (very naive approach)
    let mut current_object = String::new();
    let mut brace_count = 0;
    let mut in_string = false;

    for ch in array_content.chars() {
        if ch == '"' && (current_object.is_empty() || !current_object.ends_with('\\')) {
            in_string = !in_string;
        }

        if !in_string {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    current_object.push(ch);
                    // Parse this object
                    if let Ok(file) = GitHubFile::from_json(&current_object) {
                        files.push(file);
                    }
                    current_object.clear();
                    continue;
                }
            }
        }

        current_object.push(ch);
    }

    Ok(files)
}

impl FileDownloader {
    pub fn new() -> Self {
        let client = attohttpc::Session::new();

        Self { client }
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
        let response = self.client
            .get(&api_url)
            .send()
            .map_err(GcpError::from)?;

        if !response.status().is_success() {
            return Err(GcpError::NetworkError(format!(
                "GitHub API returned HTTP {}",
                response.status()
            )));
        }

        let response_text = response.text()
            .map_err(|e| GcpError::NetworkError(format!("Failed to read response: {}", e)))?;
        let file_info = GitHubFile::from_json(&response_text)?;

        // Try to decode base64 content directly from API response
        if let (Some(content), Some(encoding)) = (file_info.content, file_info.encoding) {
            if encoding == "base64" {
                let clean_content = content.trim().replace("\\n", "").replace("\\r", "").replace('\n', "").replace('\r', "");
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
            let response = self.client
                .get(&download_url)
                .send()
                .map_err(GcpError::from)?;

            if response.status().is_success() {
                let data = response.text()
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
        let response = self.client
            .get(&api_url)
            .send()
            .map_err(GcpError::from)?;

        if !response.status().is_success() {
            return Err(GcpError::NetworkError(format!(
                "GitHub API returned HTTP {}",
                response.status()
            )));
        }

      let response_text = response.text()
            .map_err(|e| GcpError::NetworkError(format!("Failed to read response: {}", e)))?;
        let files = parse_github_file_array(&response_text)?;

        let mut downloaded_count = 0;

        for file in files {
            let file_path = Path::new(destination).join(&file.name);

            if file.file_type == "file" {
                if let Some(download_url) = file.download_url {
                    let response = self.client
                        .get(&download_url)
                        .send()
                        .map_err(GcpError::from)?;

                    if response.status().is_success() {
                        let data = response.text()
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