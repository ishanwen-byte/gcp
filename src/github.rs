//! Minimal GitHub URL parsing for lightweight environment

use std::string::String;
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
        let parsed = url::Url::parse(url_str)?;

        match parsed.host_str() {
            Some("github.com") => Self::parse_github_url(&parsed),
            Some("raw.githubusercontent.com") => Self::parse_raw_url(&parsed),
            _ => Err(GcpError::InvalidUrl("Only GitHub URLs are supported".to_string())),
        }
    }

    fn parse_github_url(parsed: &url::Url) -> GcpResult<Self> {
        let segments: Vec<&str> = parsed.path_segments()
            .ok_or_else(|| GcpError::InvalidUrl("Invalid path".to_string()))?
            .collect();

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

    fn parse_raw_url(parsed: &url::Url) -> GcpResult<Self> {
        let segments: Vec<&str> = parsed.path_segments()
            .ok_or_else(|| GcpError::InvalidUrl("Invalid raw URL path".to_string()))?
            .collect();

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
            raw_url: parsed.to_string(),
        })
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