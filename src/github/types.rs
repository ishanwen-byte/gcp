use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRateLimit {
    pub limit: u32,
    pub remaining: u32,
    pub reset: u64,
    pub used: u32,
    pub resource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRateLimitResources {
    pub core: GitHubRateLimit,
    pub search: GitHubRateLimit,
    pub graphql: GitHubRateLimit,
    pub integration_manifest: GitHubRateLimit,
    pub code_search: GitHubRateLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRateLimitResponse {
    pub resources: GitHubRateLimitResources,
    pub rate: GitHubRateLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubFile {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: i64,
    pub url: String,
    pub html_url: String,
    pub git_url: String,
    pub download_url: Option<String>,
    #[serde(rename = "type")]
    pub file_type: String,
    pub content: Option<String>,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub fork: bool,
    pub html_url: String,
    pub default_branch: String,
    pub size: i64,
    pub stargazers_count: i64,
    pub watchers_count: i64,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubFileContent {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub content: String,
    pub encoding: String,
    pub size: i64,
    pub url: String,
    pub html_url: String,
    pub download_url: String,
    #[serde(rename = "type")]
    pub file_type: String,
}


impl GitHubFile {
    pub fn is_file(&self) -> bool {
        self.file_type == "file"
    }

    pub fn is_directory(&self) -> bool {
        self.file_type == "dir"
    }

    pub fn is_submodule(&self) -> bool {
        self.file_type == "submodule"
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type == "symlink"
    }

    pub fn get_decoded_content(&self) -> Option<Vec<u8>> {
        if let (Some(content), Some(encoding)) = (&self.content, &self.encoding) {
            if encoding == "base64" {
                use base64::{Engine as _, engine::general_purpose::STANDARD};
                STANDARD.decode(content).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}