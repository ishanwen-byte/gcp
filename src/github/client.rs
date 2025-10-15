use octocrab::Octocrab;
use tracing::{debug, info};
use std::sync::Arc;

use crate::error::{GcpError, Result};
use crate::github::{RepositoryInfo, Authentication};

#[derive(Clone)]
pub struct GitHubClient {
    pub(crate) client: Arc<Octocrab>,
    config: Arc<crate::Config>,
}

impl GitHubClient {
    pub async fn new(config: crate::Config, auth: Option<Authentication>) -> Result<Self> {
        let mut builder = Octocrab::builder();

        if let Some(ref auth) = auth {
            auth.validate_token_format()?;
            builder = builder.personal_token(auth.token.clone());
            debug!("Using GitHub authentication from source: {:?}", auth.source);
        }

        let client = builder.build()
            .map_err(|e| GcpError::GitHubApi {
                status: 0,
                message: format!("Failed to create GitHub client: {}", e),
            })?;

        // Validate authentication if provided
        if let Some(ref auth) = auth {
            auth.validate_with_github(&client).await?;
            info!("GitHub authentication validated successfully");
        }

        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
        })
    }

    pub async fn get_repository_info(&self, owner: &str, repo: &str) -> Result<RepositoryInfo> {
        let repo_info = self.client.repos(owner, repo).get().await
            .map_err(|e| GcpError::GitHubApi {
                status: 0,
                message: format!("Failed to get repository info: {}", e),
            })?;

        Ok(RepositoryInfo {
            id: repo_info.id.0 as i64,
            name: repo_info.name,
            full_name: repo_info.full_name.unwrap_or_else(|| format!("{}/{}", owner, repo)),
            description: repo_info.description,
            private: repo_info.private.unwrap_or(false),
            fork: repo_info.fork.unwrap_or(false),
            html_url: repo_info.html_url.map(|u| u.to_string()).unwrap_or_else(|| format!("https://github.com/{}/{}", owner, repo)),
            default_branch: repo_info.default_branch.unwrap_or_else(|| "main".to_string()),
            size: repo_info.size.map(|s| s as i64).unwrap_or(0),
            stargazers_count: repo_info.stargazers_count.map(|s| s as i64).unwrap_or(0),
            watchers_count: repo_info.watchers_count.map(|s| s as i64).unwrap_or(0),
            language: repo_info.language.and_then(|lang| match lang {
                serde_json::Value::String(s) => Some(s),
                _ => None,
            }),
            created_at: repo_info.created_at.unwrap_or_else(|| chrono::Utc::now()),
            updated_at: repo_info.updated_at.unwrap_or_else(|| chrono::Utc::now()),
            pushed_at: repo_info.pushed_at.unwrap_or_else(|| chrono::Utc::now()),
        })
    }

    pub async fn download_file_content(&self, url: &str) -> Result<Vec<u8>> {
        let response = reqwest::get(url).await
            .map_err(|e| GcpError::Network { source: e })?;

        if !response.status().is_success() {
            return Err(GcpError::DownloadFailed {
                file: url.to_string(),
                reason: format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown")),
            });
        }

        let bytes = response.bytes().await
            .map_err(|e| GcpError::Network { source: e })?;

        Ok(bytes.to_vec())
    }

    // Simplified content getter for MVP
    pub async fn get_file_info(&self, owner: &str, repo: &str, path: &str, ref_: Option<&str>) -> Result<(String, u64)> {
        let handler = self.client.repos(owner, repo);
        let content = handler
            .get_content()
            .path(path)
            .r#ref(ref_.unwrap_or("main"))
            .send()
            .await
            .map_err(|e| GcpError::GitHubApi {
                status: 0,
                message: format!("Failed to get file info: {}", e),
            })?;

        // Handle ContentItems which contains a vector of items
        if let Some(item) = content.items.first() {
            if let Some(decoded_content) = item.decoded_content() {
                let size: u64 = decoded_content.len() as u64;
                Ok((String::from_utf8_lossy(decoded_content.as_bytes()).to_string(), size))
            } else {
                Err(GcpError::GitHubApi {
                    status: 0,
                    message: "File content could not be decoded".to_string(),
                })
            }
        } else {
            Err(GcpError::GitHubApi {
                status: 0,
                message: "No file content found".to_string(),
            })
        }
    }
}