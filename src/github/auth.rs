use std::env;
use crate::error::{GcpError, Result};

#[derive(Debug, Clone)]
pub struct Authentication {
    pub token: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub source: AuthSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthSource {
    Environment,
    CommandLine,
    ConfigFile,
}

impl Authentication {
    pub fn from_env() -> Result<Option<Self>> {
        match env::var("GITHUB_TOKEN") {
            Ok(token) => Ok(Some(Authentication {
                token,
                scopes: vec![], // We don't know the scopes without API call
                expires_at: None,
                source: AuthSource::Environment,
            })),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(e) => Err(GcpError::Config {
                message: format!("Error reading GITHUB_TOKEN: {}", e),
            }),
        }
    }

    pub fn from_token(token: String) -> Self {
        Authentication {
            token,
            scopes: vec![], // We don't know the scopes without API call
            expires_at: None,
            source: AuthSource::CommandLine,
        }
    }

    pub fn validate_token_format(&self) -> Result<()> {
        if self.token.is_empty() {
            return Err(GcpError::Authentication {
                reason: "Token cannot be empty".to_string(),
            });
        }

        // GitHub tokens should start with "ghp_" (personal access tokens)
        // or "gho_" (oauth tokens) or "ghu_" (user tokens)
        if !self.token.starts_with("ghp_")
            && !self.token.starts_with("gho_")
            && !self.token.starts_with("ghu_")
            && !self.token.starts_with("github_pat_") {
            // Allow legacy tokens that don't start with prefixes
            tracing::warn!("GitHub token doesn't start with expected prefix (ghp_, gho_, ghu_, github_pat_)");
        }

        Ok(())
    }

    pub fn mask_token(&self) -> String {
        if self.token.len() <= 8 {
            return "*".repeat(self.token.len());
        }

        let start = &self.token[..4];
        let end = &self.token[self.token.len() - 4..];
        format!("{}...{}", start, end)
    }

    pub async fn validate_with_github(&self, client: &octocrab::Octocrab) -> Result<()> {
        // Test the token by making a simple API call
        match client.current().user().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("401") || error_msg.contains("403") {
                    Err(GcpError::Authentication {
                        reason: "Invalid or expired GitHub token".to_string(),
                    })
                } else {
                    Err(GcpError::GitHubApi {
                        status: 0,
                        message: error_msg,
                    })
                }
            }
        }
    }
}