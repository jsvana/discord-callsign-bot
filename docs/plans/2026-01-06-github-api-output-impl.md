# GitHub API Output Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace local file output with GitHub API commits for member lists.

**Architecture:** New `GitHubClient` wraps reqwest to call GitHub Contents API. `OutputConfig` changes from `file_path` to `repo/path/branch`. The `write_output_file` function becomes `generate_output_content` returning a String, which gets committed via GitHub API.

**Tech Stack:** reqwest (HTTP client), base64 (encoding), GitHub Contents API

---

### Task 1: Add Dependencies

**Files:**
- Modify: `Cargo.toml:17` (after qrz-xml line)

**Step 1: Add reqwest and base64 dependencies**

Add after the `qrz-xml` line:

```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
base64 = "0.22"
```

**Step 2: Verify dependencies resolve**

Run: `cargo check`
Expected: Compiles successfully (may take a moment to download)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "deps: add reqwest and base64 for GitHub API"
```

---

### Task 2: Update OutputConfig

**Files:**
- Modify: `src/config.rs:33-40`

**Step 1: Update OutputConfig struct**

Replace the `OutputConfig` struct:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    pub repo: String,
    pub path: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    pub default_suffix: String,
    #[serde(default = "default_emoji_separator")]
    pub emoji_separator: String,
    pub title: Option<String>,
}

fn default_branch() -> String {
    "main".to_string()
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Errors about `file_path` not found (expected - we'll fix in Task 5)

**Step 3: Commit**

```bash
git add src/config.rs
git commit -m "config: replace file_path with repo/path/branch"
```

---

### Task 3: Create GitHub Client Module

**Files:**
- Create: `src/github.rs`

**Step 1: Create the github.rs module**

```rust
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::env;

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
        let token = env::var("GITHUB_TOKEN")
            .context("GITHUB_TOKEN environment variable not set")?;

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
        let url = format!(
            "https://api.github.com/repos/{}/contents/{}",
            repo, path
        );

        // Try to get existing file SHA
        let sha = self.get_file_sha(repo, path, branch).await.ok();

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
            anyhow::bail!(
                "GitHub API returned error {}: {}",
                status,
                body
            );
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
            anyhow::bail!("File not found or API error");
        }

        let content: ContentResponse = response
            .json()
            .await
            .context("Failed to parse GitHub response")?;

        Ok(content.sha)
    }
}
```

**Step 2: Add module declaration to main.rs**

Add after line 4 (`mod qrz;`):

```rust
mod github;
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Warning about unused imports (expected - we'll use them in Task 5)

**Step 4: Commit**

```bash
git add src/github.rs src/main.rs
git commit -m "feat: add GitHub client module"
```

---

### Task 4: Update output.rs

**Files:**
- Modify: `src/output.rs`

**Step 1: Replace write_output_file with generate_output_content**

Replace the entire file:

```rust
#[derive(Debug)]
pub struct OutputEntry {
    pub callsign: String,
    pub name: String,
    pub suffix: String,
    pub emoji_separator: String,
}

pub fn generate_output_content(entries: Vec<OutputEntry>, title: Option<&str>) -> String {
    let mut output = String::new();

    // Write title header if configured
    if let Some(title_text) = title {
        output.push_str(&format!("# TITLE: {}\n", title_text));
    }

    // Sort entries by callsign for consistent output
    let mut sorted_entries = entries;
    sorted_entries.sort_by(|a, b| a.callsign.cmp(&b.callsign));

    for entry in sorted_entries {
        output.push_str(&format!(
            "{} {} {} {}\n",
            entry.callsign, entry.emoji_separator, entry.name, entry.suffix
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_output_content_basic() {
        let entries = vec![
            OutputEntry {
                callsign: "W6JSV".to_string(),
                name: "Jay".to_string(),
                suffix: "".to_string(),
                emoji_separator: "📻".to_string(),
            },
        ];

        let result = generate_output_content(entries, None);
        assert_eq!(result, "W6JSV 📻 Jay \n");
    }

    #[test]
    fn test_generate_output_content_with_title() {
        let entries = vec![
            OutputEntry {
                callsign: "W6JSV".to_string(),
                name: "Jay".to_string(),
                suffix: "".to_string(),
                emoji_separator: "📻".to_string(),
            },
        ];

        let result = generate_output_content(entries, Some("Test Title"));
        assert!(result.starts_with("# TITLE: Test Title\n"));
    }

    #[test]
    fn test_generate_output_content_sorts_by_callsign() {
        let entries = vec![
            OutputEntry {
                callsign: "KI7QCF".to_string(),
                name: "Forrest".to_string(),
                suffix: "".to_string(),
                emoji_separator: "📻".to_string(),
            },
            OutputEntry {
                callsign: "AA1AA".to_string(),
                name: "Alpha".to_string(),
                suffix: "".to_string(),
                emoji_separator: "📻".to_string(),
            },
        ];

        let result = generate_output_content(entries, None);
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[0].starts_with("AA1AA"));
        assert!(lines[1].starts_with("KI7QCF"));
    }
}
```

**Step 2: Run tests**

Run: `cargo test output`
Expected: All 3 tests pass

**Step 3: Commit**

```bash
git add src/output.rs
git commit -m "refactor: output returns String instead of writing file"
```

---

### Task 5: Update main.rs Integration

**Files:**
- Modify: `src/main.rs`

**Step 1: Update imports**

Replace line 9:
```rust
use output::{write_output_file, OutputEntry};
```

With:
```rust
use github::GitHubClient;
use output::{generate_output_content, OutputEntry};
```

**Step 2: Add GitHubClient to Handler struct**

Replace the Handler struct (lines 29-33):

```rust
struct Handler {
    config: Config,
    parser: CallsignParser,
    qrz_client: Option<Arc<QrzClient>>,
    github_client: GitHubClient,
}
```

**Step 3: Update Handler::new**

Replace the `new` function (lines 35-42):

```rust
impl Handler {
    fn new(config: Config, qrz_client: Option<Arc<QrzClient>>, github_client: GitHubClient) -> Self {
        Self {
            config,
            parser: CallsignParser::new(),
            qrz_client,
            github_client,
        }
    }
```

**Step 4: Update generate_member_list to use GitHub**

Replace lines 193-213 (the dbg! through end of function):

```rust
        info!(
            "Committing {} unique entries to GitHub (filtered {} duplicates)",
            unique_entries.len(),
            seen_callsigns.len() - unique_entries.len()
        );

        // Generate content and commit to GitHub
        let content = generate_output_content(
            unique_entries,
            guild_config.output.title.as_deref(),
        );

        self.github_client
            .commit_file(
                &guild_config.output.repo,
                &guild_config.output.path,
                &guild_config.output.branch,
                &content,
                "Update member list",
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to commit to GitHub: {}", e))?;

        info!(
            "Successfully committed member list to {}/{}",
            guild_config.output.repo, guild_config.output.path
        );

        Ok(())
    }
}
```

**Step 5: Update main function to initialize GitHubClient**

Add after QRZ client initialization (after line 375, before Discord client setup):

```rust
    // Initialize GitHub client
    info!("Initializing GitHub client...");
    let github_client = GitHubClient::new()?;
    info!("GitHub client initialized successfully");
```

**Step 6: Update Handler construction**

Replace line 381:
```rust
        .event_handler(Handler::new(config, qrz_client))
```

With:
```rust
        .event_handler(Handler::new(config, qrz_client, github_client))
```

**Step 7: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully

**Step 8: Commit**

```bash
git add src/main.rs
git commit -m "feat: integrate GitHub client for member list commits"
```

---

### Task 6: Update Example Config

**Files:**
- Modify: `config.example.toml` (if exists, otherwise create)

**Step 1: Check if example config exists**

Run: `ls config*.toml 2>/dev/null || echo "no example config"`

**Step 2: Create/update example config**

Create `config.example.toml`:

```toml
[discord]
token = "your-discord-bot-token"

# Optional: QRZ credentials for operator name lookups
# [qrz]
# username = "your-qrz-username"
# password = "your-qrz-password"

[[guilds]]
guild_id = 123456789012345678
bot_nickname = "Callsign Bot"

[guilds.output]
repo = "username/repo-name"
path = "members/guild-name.txt"
branch = "main"
default_suffix = ""
emoji_separator = "📻"
title = "Guild Member List"

# Optional: Per-user overrides
# [guilds.overrides."discord-user-id"]
# callsign = "W1AW"
# name = "ARRL HQ"
# suffix = "(Special)"
```

**Step 3: Commit**

```bash
git add config.example.toml
git commit -m "docs: update example config for GitHub output"
```

---

### Task 7: Run Full Test Suite and Verify

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 3: Check formatting**

Run: `cargo fmt -- --check`
Expected: No formatting issues

**Step 4: Final commit if any fixes needed**

If clippy or fmt required changes:
```bash
git add -A
git commit -m "chore: fix clippy warnings and formatting"
```
