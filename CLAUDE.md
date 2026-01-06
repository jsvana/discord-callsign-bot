# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust-based Discord bot that automatically generates and maintains formatted member lists of amateur radio operators. The bot:
- Monitors multiple Discord servers (guilds) simultaneously
- Parses callsigns from member names using regex patterns
- Optionally looks up operator information from QRZ.com
- Generates member lists committed to GitHub repositories
- Updates in real-time when members join/leave or update their profiles

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Build optimized release binary
cargo build --release

# Run the bot
cargo run

# Run with custom config path
CONFIG_PATH=/path/to/config.toml cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Run with GitHub token (required for output)
GITHUB_TOKEN=your_token_here cargo run
```

### Testing and Quality
```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test parser
cargo test qrz

# Check code formatting
cargo fmt -- --check

# Format code
cargo fmt

# Run clippy linter
cargo clippy -- -D warnings
```

### Git Hooks
The repository has pre-commit hooks that run `cargo fmt`, `cargo build`, and `cargo clippy`. Install them with:
```bash
./scripts/install-git-hooks.sh
```

## Architecture

### Module Structure

The codebase is organized into 6 modules:

- **main.rs**: Discord bot event handler and orchestration
  - `Handler` struct owns config, parser, QRZ client, and GitHub client
  - Implements `EventHandler` trait for Discord events (ready, member_addition, member_removal, member_update)
  - `generate_member_list()` is the core function that processes members for a guild

- **config.rs**: Configuration management
  - Loads TOML config from file (default: `config.toml`)
  - Supports multiple guild configurations with per-guild overrides
  - Each guild has separate output settings (repo, path, branch, suffix, emoji, title)
  - User overrides are keyed by Discord user ID and are per-guild

- **parser.rs**: Callsign parsing logic
  - `CallsignParser` uses regex to find callsigns: `[A-Z0-9]{1,2}[0-9][A-Z]{1,4}`
  - `parse()` method extracts callsign and name from various formats:
    - "W6JSV - Jay"
    - "Forrest KI7QCF"
    - "Jay (w6jsv)"
    - "W6JSV" (callsign-only)
  - Case-insensitive matching with uppercase normalization

- **qrz.rs**: QRZ.com integration
  - `QrzClient` wraps the qrz-xml crate
  - Authenticates with QRZ credentials on initialization
  - `lookup_callsign()` fetches operator info
  - `get_display_name()` prioritizes: nickname → fname → name

- **github.rs**: GitHub API integration
  - `GithubClient` handles authentication and API calls
  - Uses `GITHUB_TOKEN` environment variable for authentication
  - `commit_file()` creates or updates files in GitHub repositories
  - Supports specifying target repository, file path, and branch

- **output.rs**: Output content generation
  - `generate_output_content()` returns formatted String content
  - Format: `<CALLSIGN> <EMOJI> <NAME> <SUFFIX>`
  - Optional title header: `# TITLE: <title>`
  - Entries are sorted alphabetically by callsign

### Key Data Flow

1. **Bot startup** (`ready` event):
   - Load config from TOML file
   - Initialize optional QRZ client if credentials provided
   - For each configured guild:
     - Set bot nickname if configured
     - Call `generate_member_list()`

2. **Member list generation** (`generate_member_list`):
   - Fetch all members from guild via Discord API
   - For each member (skipping bot itself):
     - Try parsing callsign from: nickname → global_name → username (in priority order)
     - Check for manual override in config (by Discord user ID)
     - If QRZ client available, lookup operator name
     - Create `OutputEntry` with callsign, name, suffix, emoji
   - Deduplicate by callsign (keep first occurrence)
   - Commit sorted entries to configured GitHub repository

3. **Real-time updates** (event handlers):
   - `guild_member_addition`: Regenerate list when member joins
   - `guild_member_removal`: Regenerate list when member leaves
   - `guild_member_update`: Regenerate list when member profile changes

### Configuration Structure

The TOML config supports:
- Single Discord bot token (shared across all guilds)
- Optional QRZ credentials (shared across all guilds)
- `GITHUB_TOKEN` environment variable (required for committing output)
- Array of guild configs (`[[guilds]]`), each with:
  - `guild_id`: Discord server ID (u64)
  - `bot_nickname`: Optional custom nickname for this guild
  - `output`: GitHub repo, file path, branch, default suffix, emoji separator, optional title
  - `overrides`: HashMap of Discord user IDs to override settings (callsign, name, suffix, emoji)

### Multi-Guild Support

The bot monitors multiple Discord servers simultaneously. Each guild has:
- Independent GitHub output (repo, path, branch)
- Separate default suffix and emoji settings
- Per-guild user overrides (same user can have different callsigns/names on different servers)
- Optional per-guild bot nickname

The `Config::get_guild_config()` method looks up guild config by ID, and event handlers only process configured guilds.

### Discord Event Handler Notes

- Uses serenity 0.12 with rustls backend
- Required gateway intents: `GUILDS` and `GUILD_MEMBERS`
- Bot needs "SERVER MEMBERS INTENT" enabled in Discord Developer Portal
- Events are async and use tokio runtime
- Member data includes: nick (server nickname), global_name, username - checked in that priority order

## Testing

Tests are inline in each module using `#[cfg(test)]`:
- parser.rs: Tests various callsign formats and case handling
- qrz.rs: Tests display name priority logic
- No integration tests currently; all tests are unit tests

Run tests with `cargo test` or test specific modules with `cargo test <module_name>`.
