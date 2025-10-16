//! Minimal GitHub URL parsing for lightweight environment

use crate::error::{GcpError, GcpResult};

#[derive(Debug, Clone, PartialEq)]
pub enum UrlType {
    File,
    Folder,
    Repository,
}

#[derive(Debug, Clone)]
pub struct GitHubUrl {
    pub owner: String,
    pub repo: String,
    pub path: Option<String>,
    pub ref_: Option<String>,
    pub url_type: UrlType,
    pub raw_url: String,
}

impl GitHubUrl {
    pub fn parse(url_str: &str) -> GcpResult<Self> {
        // Basic URL validation
        if !url_str.starts_with("https://") {
            return Err(GcpError::InvalidUrl("Only HTTPS URLs are supported".to_string()));
        }

        // Extract host
        let after_protocol = &url_str[8..];
        let host_end = after_protocol.find('/').unwrap_or(after_protocol.len());
        let host = &after_protocol[..host_end];

        // Parse path
        let path = &after_protocol[host_end..];
        let path = path.trim_start_matches('/');

        match host {
            "github.com" => Self::parse_github_url(path),
            "raw.githubusercontent.com" => Self::parse_raw_url(path, url_str),
            _ => Err(GcpError::InvalidUrl("Only GitHub URLs are supported".to_string())),
        }
    }

    fn parse_github_url(path: &str) -> GcpResult<Self> {
        let segments: Vec<&str> = path.split('/').collect();

        if segments.len() < 2 {
            return Err(GcpError::InvalidUrl("Invalid GitHub URL format".to_string()));
        }

        let owner = segments[0].to_string();
        let repo = segments[1].to_string();

        if segments.len() >= 4 {
            let indicator = segments[2];
            let ref_ = Some(segments[3].to_string());
            let path = if segments.len() > 4 {
                Some(segments[4..].join("/"))
            } else {
                None
            };

            let url_type = match indicator {
                "blob" => UrlType::File,
                "tree" => UrlType::Folder,
                _ => return Err(GcpError::InvalidUrl("Invalid GitHub URL type".to_string())),
            };

            let raw_url = if let Some(ref path) = path {
                format!(
                    "https://raw.githubusercontent.com/{}/{}/{}/{}",
                    owner, repo, ref_.as_deref().unwrap_or("main"), path
                )
            } else {
                format!(
                    "https://raw.githubusercontent.com/{}/{}/{}",
                    owner, repo, ref_.as_deref().unwrap_or("main")
                )
            };

            Ok(GitHubUrl {
                owner,
                repo,
                path,
                ref_,
                url_type,
                raw_url,
            })
        } else {
            // Repository URL - not supported in minimal version
            Ok(GitHubUrl {
                owner,
                repo,
                path: None,
                ref_: None,
                url_type: UrlType::Repository,
                raw_url: String::new(),
            })
        }
    }

    fn parse_raw_url(path: &str, original_url: &str) -> GcpResult<Self> {
        let segments: Vec<&str> = path.split('/').collect();

        if segments.len() < 3 {
            return Err(GcpError::InvalidUrl("Invalid raw GitHub URL format".to_string()));
        }

        let owner = segments[0].to_string();
        let repo = segments[1].to_string();
        let ref_ = Some(segments[2].to_string());
        let path = if segments.len() > 3 {
            Some(segments[3..].join("/"))
        } else {
            None
        };

        let url_type = if path.is_some() { UrlType::File } else { UrlType::Repository };

        Ok(GitHubUrl {
            owner,
            repo,
            path,
            ref_,
            url_type,
            raw_url: original_url.to_string(),
        })
    }

    /// Extract filename from path for file URLs
    pub fn filename(&self) -> Option<String> {
        match self.url_type {
            UrlType::File => {
                self.path.as_ref().and_then(|path| {
                    path.split('/').last().map(|filename| filename.to_string())
                })
            }
            UrlType::Folder => {
                self.path.as_ref().and_then(|path| {
                    path.split('/').last().map(|foldername| foldername.to_string())
                })
            }
            UrlType::Repository => None,
        }
    }

    pub fn api_url(&self) -> String {
        match self.url_type {
            UrlType::File | UrlType::Folder => {
                format!(
                    "https://api.github.com/repos/{}/{}/contents/{}",
                    self.owner,
                    self.repo,
                    self.path.as_deref().unwrap_or("")
                )
            }
            UrlType::Repository => {
                format!("https://api.github.com/repos/{}/{}", self.owner, self.repo)
            }
        }
    }
}