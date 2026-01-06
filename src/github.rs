use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, warn};

pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
}

#[derive(Deserialize)]
struct ContentResponse {
    sha: String,
}

#[derive(Serialize)]
struct UpdateFileRequest<'a> {
    message: &'a str,
    content: &'a str,
    branch: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<&'a str>,
}

impl GitHubClient {
    pub fn new() -> Result<Self> {
        let token =
            env::var("GITHUB_TOKEN").context("GITHUB_TOKEN environment variable not set")?;

        let client = reqwest::Client::new();

        Ok(Self { client, token })
    }

    pub async fn commit_file(
        &self,
        repo: &str,
        path: &str,
        branch: &str,
        content: &str,
        message: &str,
    ) -> Result<()> {
        let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);

        // Try to get existing file SHA (404 is expected for new files)
        let sha = match self.get_file_sha(repo, path, branch).await {
            Ok(sha) => Some(sha),
            Err(e) => {
                warn!(
                    "Could not get file SHA for {}/{}: {} (file may not exist yet)",
                    repo, path, e
                );
                None // File doesn't exist yet, will create
            }
        };

        info!("Committing to {}/{} on branch {}", repo, path, branch);

        // Base64 encode the content
        let encoded_content = STANDARD.encode(content);

        let request_body = UpdateFileRequest {
            message,
            content: &encoded_content,
            branch,
            sha: sha.as_deref(),
        };

        let response = self
            .client
            .put(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(USER_AGENT, "discord-callsign-bot")
            .header(ACCEPT, "application/vnd.github+json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API returned error {}: {}", status, body);
        }

        Ok(())
    }

    async fn get_file_sha(&self, repo: &str, path: &str, branch: &str) -> Result<String> {
        let url = format!(
            "https://api.github.com/repos/{}/contents/{}?ref={}",
            repo, path, branch
        );

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(USER_AGENT, "discord-callsign-bot")
            .header(ACCEPT, "application/vnd.github+json")
            .send()
            .await
            .context("Failed to fetch file from GitHub")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "GitHub API error (status {}): file not found or access denied",
                response.status()
            );
        }

        let content: ContentResponse = response
            .json()
            .await
            .context("Failed to parse GitHub response")?;

        Ok(content.sha)
    }
}
