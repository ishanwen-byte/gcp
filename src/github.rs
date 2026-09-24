//! Minimal GitHub/Gitea URL parsing for lightweight environment

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
    /// API base endpoint: "https://api.github.com" for GitHub,
    /// "http(s)://<gitea-host>/api/v1" for Gitea-compatible servers.
    pub api_base: String,
    /// "https" or "http" (intranet Gitea may be plain http)
    pub scheme: String,
    /// Original web host (e.g. "github.com" or "192.168.3.14:3000")
    pub web_host: String,
}

impl GitHubUrl {
    pub fn parse(url_str: &str) -> GcpResult<Self> {
        // Extract scheme
        let (scheme, after_protocol) = if let Some(rest) = url_str.strip_prefix("https://") {
            ("https", rest)
        } else if let Some(rest) = url_str.strip_prefix("http://") {
            // Plain http is allowed for intranet Gitea instances
            ("http", rest)
        } else {
            return Err(GcpError::InvalidUrl(
                "Only HTTP(S) URLs are supported".to_string(),
            ));
        };

        // Extract host (may include :port)
        let host_end = after_protocol.find('/').unwrap_or(after_protocol.len());
        let host = &after_protocol[..host_end];

        // Parse path
        let path = &after_protocol[host_end..];
        let path = path.trim_start_matches('/');

        let api_base;
        match host {
            "github.com" => {
                api_base = "https://api.github.com".to_string();
                Self::parse_repo_url(
                    path,
                    api_base,
                    scheme,
                    host,
                    "https://raw.githubusercontent.com/",
                )
            }
            "raw.githubusercontent.com" => {
                api_base = "https://api.github.com".to_string();
                Self::parse_raw_url(path, url_str, api_base, scheme, host)
            }
            _ => {
                // Gitea-compatible server: <scheme>://<host>/<owner>/<repo>/...
                api_base = format!("{}://{}/api/v1", scheme, host);
                Self::parse_repo_url(
                    path,
                    api_base,
                    scheme,
                    host,
                    &format!("{}://{}/", scheme, host),
                )
            }
        }
    }

    fn parse_repo_url(
        path: &str,
        api_base: String,
        scheme: &str,
        web_host: &str,
        raw_base: &str,
    ) -> GcpResult<Self> {
        let segments: Vec<&str> = path.split('/').collect();

        if segments.len() < 2 {
            return Err(GcpError::InvalidUrl(
                "Invalid repository URL format".to_string(),
            ));
        }

        let owner = segments[0].to_string();
        let repo = segments[1].to_string();

        if segments.len() >= 4 {
            let indicator = segments[2];
            let ref_ = Some(segments[3].to_string());
            let sub_path = if segments.len() > 4 {
                Some(segments[4..].join("/"))
            } else {
                None
            };

            let url_type = match indicator {
                "blob" => UrlType::File,
                "tree" => UrlType::Folder,
                // Gitea web URLs use /src/ for both files and folders
                "src" => {
                    if let Some(p) = &sub_path {
                        // Heuristic: presence of a dot in the last segment
                        // suggests a file. The API response will correct us
                        // either way (type field).
                        let last = p.rsplit('/').next().unwrap_or(p);
                        if last.contains('.') {
                            UrlType::File
                        } else {
                            UrlType::Folder
                        }
                    } else {
                        UrlType::Folder
                    }
                }
                _ => {
                    return Err(GcpError::InvalidUrl(
                        "Invalid repository URL type (expected blob/tree/src)".to_string(),
                    ));
                }
            };

            let raw_url = if let Some(ref p) = sub_path {
                format!(
                    "{}{}/{}/{}/{}",
                    raw_base,
                    owner,
                    repo,
                    ref_.as_deref().unwrap_or("main"),
                    p
                )
            } else {
                format!(
                    "{}{}/{}/{}",
                    raw_base,
                    owner,
                    repo,
                    ref_.as_deref().unwrap_or("main")
                )
            };

            Ok(GitHubUrl {
                owner,
                repo,
                path: sub_path,
                ref_,
                url_type,
                raw_url,
                api_base,
                scheme: scheme.to_string(),
                web_host: web_host.to_string(),
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
                api_base,
                scheme: scheme.to_string(),
                web_host: web_host.to_string(),
            })
        }
    }

    fn parse_raw_url(
        path: &str,
        original_url: &str,
        api_base: String,
        scheme: &str,
        web_host: &str,
    ) -> GcpResult<Self> {
        let segments: Vec<&str> = path.split('/').collect();

        if segments.len() < 3 {
            return Err(GcpError::InvalidUrl(
                "Invalid raw GitHub URL format".to_string(),
            ));
        }

        let owner = segments[0].to_string();
        let repo = segments[1].to_string();
        let ref_ = Some(segments[2].to_string());
        let sub_path = if segments.len() > 3 {
            Some(segments[3..].join("/"))
        } else {
            None
        };

        let url_type = if sub_path.is_some() {
            UrlType::File
        } else {
            UrlType::Repository
        };

        Ok(GitHubUrl {
            owner,
            repo,
            path: sub_path,
            ref_,
            url_type,
            raw_url: original_url.to_string(),
            api_base,
            scheme: scheme.to_string(),
            web_host: web_host.to_string(),
        })
    }

    /// Extract filename from path for file URLs
    pub fn filename(&self) -> Option<String> {
        let last_segment = |path: &Option<String>| {
            path.as_ref()
                .and_then(|p| p.rsplit('/').next().map(|s| s.to_string()))
        };
        match self.url_type {
            UrlType::File | UrlType::Folder => last_segment(&self.path),
            UrlType::Repository => None,
        }
    }

    pub fn api_url(&self) -> String {
        match self.url_type {
            UrlType::File | UrlType::Folder => {
                let base = format!(
                    "{}/repos/{}/{}/contents/{}",
                    self.api_base,
                    self.owner,
                    self.repo,
                    self.path.as_deref().unwrap_or("")
                );
                match &self.ref_ {
                    Some(r) if !r.is_empty() => format!("{}?ref={}", base, r),
                    _ => base,
                }
            }
            UrlType::Repository => {
                format!("{}/repos/{}/{}", self.api_base, self.owner, self.repo)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_file() {
        let u = GitHubUrl::parse("https://github.com/o/r/blob/main/src/a.rs").unwrap();
        assert_eq!(u.owner, "o");
        assert_eq!(u.repo, "r");
        assert_eq!(u.url_type, UrlType::File);
        assert_eq!(u.api_base, "https://api.github.com");
        assert_eq!(
            u.api_url(),
            "https://api.github.com/repos/o/r/contents/src/a.rs?ref=main"
        );
    }

    #[test]
    fn test_parse_gitea_file() {
        let u = GitHubUrl::parse("http://192.168.3.14:3000/goliath/repo/src/main/docs/readme.md")
            .unwrap();
        assert_eq!(u.owner, "goliath");
        assert_eq!(u.repo, "repo");
        assert_eq!(u.url_type, UrlType::File);
        assert_eq!(u.api_base, "http://192.168.3.14:3000/api/v1");
        assert_eq!(
            u.api_url(),
            "http://192.168.3.14:3000/api/v1/repos/goliath/repo/contents/docs/readme.md?ref=main"
        );
    }

    #[test]
    fn test_parse_gitea_folder() {
        let u = GitHubUrl::parse("http://192.168.3.14:3000/goliath/repo/src/main/docs").unwrap();
        assert_eq!(u.url_type, UrlType::Folder);
        assert_eq!(
            u.api_url(),
            "http://192.168.3.14:3000/api/v1/repos/goliath/repo/contents/docs?ref=main"
        );
    }

    #[test]
    fn test_parse_raw_github() {
        let u = GitHubUrl::parse("https://raw.githubusercontent.com/o/r/main/a.txt").unwrap();
        assert_eq!(u.url_type, UrlType::File);
        assert_eq!(u.ref_.as_deref(), Some("main"));
    }

    #[test]
    fn test_rejects_non_http() {
        assert!(GitHubUrl::parse("ftp://x.com/a/b").is_err());
        assert!(GitHubUrl::parse("github.com/o/r").is_err());
    }
}
