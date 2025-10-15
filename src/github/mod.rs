pub mod auth;
pub mod client;
pub mod types;

pub use auth::{Authentication, AuthSource};
pub use client::GitHubClient;
pub use types::{GitHubFile, RepositoryInfo, GitHubFileContent, GitHubRateLimitResponse};

use crate::error::{GcpError, Result};

#[derive(Debug, Clone)]
pub struct GitHubUrl {
    pub owner: String,
    pub repo: String,
    pub path: Option<String>,
    pub ref_: Option<String>,
    pub url_type: UrlType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UrlType {
    File,
    Folder,
    Repository,
}

impl GitHubUrl {
    pub fn parse(url: &str) -> Result<Self> {
        let parsed_url = url::Url::parse(url)?;

        // Handle different URL formats
        if parsed_url.host_str() == Some("raw.githubusercontent.com") {
            // Raw URL format: https://raw.githubusercontent.com/owner/repo/ref/path
            Self::parse_raw_url(&parsed_url)
        } else if parsed_url.host_str() == Some("github.com") {
            // GitHub URL format: https://github.com/owner/repo/blob/ref/path
            Self::parse_github_url(&parsed_url)
        } else {
            return Err(GcpError::InvalidUrl {
                url: url.to_string(),
            });
        }
    }

    fn parse_raw_url(parsed_url: &url::Url) -> Result<Self> {
        let path_segments: Vec<&str> = parsed_url.path_segments()
            .ok_or_else(|| GcpError::InvalidUrl {
                url: parsed_url.to_string(),
            })?
            .collect();

        if path_segments.len() < 3 {
            return Err(GcpError::InvalidUrl {
                url: parsed_url.to_string(),
            });
        }

        let owner = path_segments[0].to_string();
        let repo = path_segments[1].to_string();
        let ref_ = path_segments[2].to_string();
        let path = if path_segments.len() > 3 {
            Some(path_segments[3..].join("/"))
        } else {
            None
        };

        let url_type = if path.is_some() { UrlType::File } else { UrlType::Repository };

        Ok(GitHubUrl {
            owner,
            repo,
            path,
            ref_: Some(ref_),
            url_type,
        })
    }

    fn parse_github_url(parsed_url: &url::Url) -> Result<Self> {
        let path_segments: Vec<&str> = parsed_url.path_segments()
            .ok_or_else(|| GcpError::InvalidUrl {
                url: parsed_url.to_string(),
            })?
            .collect();

        if path_segments.len() < 2 {
            return Err(GcpError::InvalidUrl {
                url: parsed_url.to_string(),
            });
        }

        let owner = path_segments[0].to_string();
        let repo = path_segments[1].to_string();

        // Check for blob/tree indicators
        if path_segments.len() >= 4 {
            let indicator = path_segments[2];
            let ref_ = path_segments[3].to_string();
            let path = if path_segments.len() > 4 {
                Some(path_segments[4..].join("/"))
            } else {
                None
            };

            let url_type = match indicator {
                "blob" => UrlType::File,
                "tree" => UrlType::Folder,
                _ => return Err(GcpError::InvalidUrl {
                    url: parsed_url.to_string(),
                }),
            };

            Ok(GitHubUrl {
                owner,
                repo,
                path,
                ref_: Some(ref_),
                url_type,
            })
        } else {
            // Repository root URL
            Ok(GitHubUrl {
                owner,
                repo,
                path: None,
                ref_: None,
                url_type: UrlType::Repository,
            })
        }
    }

    pub fn api_path(&self) -> String {
        match self.url_type {
            UrlType::File | UrlType::Folder => {
                format!(
                    "/repos/{}/{}/contents/{}",
                    self.owner,
                    self.repo,
                    self.path.as_deref().unwrap_or("")
                )
            }
            UrlType::Repository => {
                format!("/repos/{}/{}", self.owner, self.repo)
            }
        }
    }

    pub fn raw_url(&self) -> Option<String> {
        match self.url_type {
            UrlType::File => Some(format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                self.owner,
                self.repo,
                self.ref_.as_deref().unwrap_or("main"),
                self.path.as_deref().unwrap_or("")
            )),
            _ => None,
        }
    }
}