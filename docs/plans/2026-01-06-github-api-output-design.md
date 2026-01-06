# GitHub API Output Design

Replace local file output with GitHub API commits.

## Decisions

- **Library**: reqwest + GitHub REST API directly (no octocrab dependency)
- **Configuration**: Replace `file_path` with `repo`, `path`, `branch` in OutputConfig
- **Authentication**: `GITHUB_TOKEN` environment variable only
- **Commit behavior**: Always commit on member change (GitHub handles no-ops for identical content)
- **Error handling**: Log errors and continue, bot stays running

## Configuration Changes

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    pub repo: String,           // e.g., "jsvana/callsign-lists"
    pub path: String,           // e.g., "members.txt"
    #[serde(default = "default_branch")]
    pub branch: String,         // e.g., "main"
    pub default_suffix: String,
    #[serde(default = "default_emoji_separator")]
    pub emoji_separator: String,
    pub title: Option<String>,
}
```

Example config:
```toml
[[guilds]]
guild_id = 123456789

[guilds.output]
repo = "jsvana/callsign-lists"
path = "guild-name/members.txt"
branch = "main"
default_suffix = ""
emoji_separator = "📻"
```

## GitHub Client Module

New `github.rs` module:

```rust
pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
}

impl GitHubClient {
    pub fn new() -> Result<Self>  // Reads GITHUB_TOKEN from env

    pub async fn commit_file(
        &self,
        repo: &str,      // "owner/repo"
        path: &str,      // file path in repo
        branch: &str,    // target branch
        content: &str,   // file content
        message: &str,   // commit message
    ) -> Result<()>
}
```

The `commit_file` method:
1. GET `/repos/{owner}/{repo}/contents/{path}?ref={branch}` to fetch current file SHA
2. PUT `/repos/{owner}/{repo}/contents/{path}` with base64-encoded content, SHA (if updating), branch, and commit message

## Integration

Handler struct adds `github_client: GitHubClient`.

`output.rs` changes:
- `write_output_file` becomes `generate_output_content` returning `String`
- Sorting and formatting logic unchanged

In `generate_member_list`:
```rust
let content = generate_output_content(entries, guild_config.output.title.as_deref());

if let Err(e) = self.github_client.commit_file(
    &guild_config.output.repo,
    &guild_config.output.path,
    &guild_config.output.branch,
    &content,
    "Update member list",
).await {
    tracing::error!("Failed to commit member list for guild {}: {}", guild_id, e);
}
```

## Dependencies

```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
base64 = "0.22"
```

## Error Handling

- **Missing `GITHUB_TOKEN`**: Bot fails to start with clear error
- **401/403/404**: Logged, bot continues, retries on next member change
- **Network errors**: Logged, bot continues
