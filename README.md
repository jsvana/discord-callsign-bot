# Discord Callsign Bot

A Rust-based Discord bot that automatically generates and maintains a formatted list of amateur radio operators from your Discord server. The bot intelligently parses callsigns from member names, optionally looks up operator information from QRZ.com, and outputs a clean, sorted text file perfect for sharing with your radio club or field day operations.

## What Does This Bot Do?

This bot helps amateur radio clubs and groups manage their member rosters by:

1. **Scanning your Discord servers** (supports multiple servers!) for members with callsigns in their names
2. **Automatically parsing callsigns** from various name formats (nicknames, display names, usernames)
3. **Looking up operator names** from QRZ.com's callbook database (optional)
4. **Generating formatted text files** with callsigns, names, and custom suffixes (separate file per server)
5. **Updating in real-time** when members join, leave, or update their profiles

Perfect for:
- Amateur radio clubs maintaining member lists
- Field day operations and portable activations
- POTA/SOTA activation groups
- Emergency communications teams
- Any ham radio community on Discord

## Features

- **Smart Callsign Detection**:
  - Checks multiple Discord name fields (nickname → global name → username)
  - Parses callsigns from various formats:
    - `W6JSV - Jay` → Callsign: W6JSV, Name: Jay
    - `Forrest KI7QCF` → Callsign: KI7QCF, Name: Forrest
    - `Jay (w6jsv)` → Callsign: W6JSV, Name: Jay
  - Case-insensitive matching with automatic uppercase normalization
  - Supports callsign-only names

- **QRZ.com Integration** (Optional):
  - Automatically looks up operator names and nicknames
  - Falls back to Discord names if QRZ lookup fails
  - Prioritizes nickname → first name → last name from QRZ data

- **Real-Time Updates**:
  - Regenerates the member list when users join or leave
  - Updates when members change their nicknames

- **Deduplication**: Ensures each callsign appears only once in the output

- **Manual Overrides**: Configure specific callsigns, names, suffix text, or emoji separators for individual users via TOML configuration

- **Configurable Output**: Customize emoji separators, default suffix text, and file titles

- **Sorted Output**: Members are sorted alphabetically by callsign in the output file

## Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- A Discord Bot Token (see Setup section)

## Setup

### 1. Create a Discord Bot

1. Go to https://discord.com/developers/applications
2. Click "New Application" and give it a name
3. Go to the "Bot" section and click "Add Bot"
4. Under "Privileged Gateway Intents", enable:
   - SERVER MEMBERS INTENT (required to read member list)
   - PRESENCE INTENT
5. Click "Reset Token" and copy your bot token
6. Go to "OAuth2" → "URL Generator"
   - Select scopes: `bot`
   - Select permissions: `Read Messages/View Channels`
7. Copy the generated URL and open it in your browser to invite the bot to your server

### 2. Get Your Guild (Server) IDs

1. In Discord, go to User Settings → Advanced → Enable "Developer Mode"
2. Right-click on your server icon and click "Copy Server ID"
3. Repeat for each server you want the bot to monitor

### 3. Configure the Bot

```bash
# Copy the example configuration
cp config.toml.example config.toml

# Edit the configuration file
nano config.toml
```

Update the following fields:
- `discord.token`: Your bot token from step 1
- For each server you want to monitor, add a `[[guilds]]` entry with:
  - `guild_id`: Your server ID from step 2
  - `guilds.output.file_path`: Where to save the output file for this server (e.g., `members.txt`)
  - `guilds.output.default_suffix`: Text to append after each entry (e.g., "73")

You can configure multiple servers - see the example in the configuration file!

### 4. Add Manual Overrides (Optional)

To override callsign/name/suffix/emoji for specific users (per-server):

1. In Discord, right-click on a user and select "Copy User ID"
2. Add an override section under the appropriate guild in `config.toml`:

```toml
[[guilds]]
guild_id = 123456789

[guilds.output]
file_path = "members.txt"
# ... other settings ...

[guilds.overrides."USER_ID_HERE"]
callsign = "W1ABC"  # Optional: override callsign
name = "John"       # Optional: override name
suffix = "CQ CQ"    # Optional: override suffix text
emoji = "✨"        # Optional: override emoji separator
```

Note: Overrides are per-server, so the same user can have different callsigns/names on different servers!

## Usage

### Build and Run

```bash
# Build the project
cargo build --release

# Run the bot
cargo run --release
```

The bot will:
1. Connect to Discord
2. Fetch all members from the configured server
3. Parse callsigns from multiple name fields (nickname, global name, username)
4. Look up names from QRZ.com (if configured)
5. Generate the output file
6. Stay running and monitor for member changes
7. Automatically regenerate the file when members join, leave, or update their profiles

**Note**: The bot runs continuously to keep the member list up-to-date. Press Ctrl+C to stop it.

### Output Format

The generated file will have one line per member:
```
<CALLSIGN> <EMOJI_SEPARATOR> <NAME> <SUFFIX>
```

Example output (`members.txt`):
```
# TITLE: Club members
KI7QCF 📻 Forrest 73
W1ABC 🌊 John CQ CQ
W6JSV ✨ Jay 73
```

- Callsigns are automatically converted to uppercase
- Entries are sorted alphabetically by callsign
- Duplicate callsigns are automatically filtered out
- The emoji separator can be customized in the config (default: 📻)

### Running with Custom Config Path

```bash
CONFIG_PATH=/path/to/config.toml cargo run --release
```

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --release
```

## Building for Production

```bash
# Build optimized binary
cargo build --release

# The binary will be at: target/release/discord-callsign-bot

# Run it
./target/release/discord-callsign-bot
```

## Docker Deployment

```bash
# Build the Docker image
docker build -t discord-callsign-bot .

# Run with mounted config
docker run -v $(pwd)/config.toml:/app/config.toml discord-callsign-bot
```

## Configuration Reference

### `[discord]`
- `token` (required): Your Discord bot token

### `[qrz]` (Optional)
Enable QRZ.com callbook lookups for automatic name retrieval (shared across all servers):
- `username` (required if using QRZ): Your QRZ.com username
- `password` (required if using QRZ): Your QRZ.com password

**Note**: Requires a QRZ.com XML subscription (https://www.qrz.com/i/subscriptions.html)

To disable QRZ lookups, simply comment out or remove the entire `[qrz]` section.

### `[[guilds]]` (Array - add one per server)
Each `[[guilds]]` entry configures monitoring for one Discord server:
- `guild_id` (required): The Discord server ID to read members from
- `bot_nickname` (optional): Set a custom nickname for the bot on this server

### `[guilds.output]`
Output configuration for each server:
- `file_path` (required): Path where the member list will be saved for this server
- `default_suffix` (required): Default text appended after each member entry
- `emoji_separator` (optional): Emoji or text between callsign and name (default: "📻")
- `title` (optional): Title header for the output file

### `[guilds.overrides."USER_ID"]`
Per-server user overrides. All fields are optional. Only specify what you want to override:
- `callsign`: Override the parsed callsign
- `name`: Override the parsed name
- `suffix`: Override the default suffix for this user
- `emoji`: Override the emoji separator for this user

**Note**: Overrides are per-server, allowing different settings for the same user across different servers.

## Troubleshooting

### Bot can't see members
- Ensure "SERVER MEMBERS INTENT" is enabled in the Discord Developer Portal
- Make sure the bot has permission to view members in your server

### No callsigns found
- Check that member display names contain valid amateur radio callsigns
- Valid formats: W6JSV, KI7QCF, N0CALL, etc. (case-insensitive)
- Callsigns must follow the pattern: `[PREFIX][DIGIT][SUFFIX]`
- The bot checks nickname, global name, and username in that order

### QRZ lookups failing
- Verify your QRZ username and password are correct in `config.toml`
- Ensure you have an active QRZ XML subscription
- Check the logs for specific error messages (run with `RUST_LOG=info`)
- The bot will fall back to Discord names if QRZ lookups fail

### Duplicate callsigns
- The bot automatically deduplicates entries, keeping the first occurrence
- Check the logs for warnings about duplicate callsigns
- Users with the same callsign will only appear once in the output

### Permission denied errors
- Ensure the output directory is writable
- Check file system permissions for the configured `output.file_path`

## License

MIT

## Contributing

Pull requests welcome! Please ensure code is formatted with `cargo fmt` and passes `cargo clippy` before submitting.

### Setting Up Git Hooks

To automatically check code quality before committing, install the git hooks:

```bash
./scripts/install-git-hooks.sh
```

This will set up a pre-commit hook that runs:
1. `cargo fmt -- --check` - Ensures code is properly formatted
2. `cargo build` - Verifies the code compiles
3. `cargo clippy -- -D warnings` - Checks for common mistakes and style issues

If any check fails, the commit will be blocked. To bypass the hooks (not recommended), use:
```bash
git commit --no-verify
```

## Amateur Radio Info

This bot recognizes standard amateur radio callsign formats used worldwide. Common prefixes include:
- US: W, K, N, A
- Canada: VE, VA, VO
- UK: G, M
- And many more international prefixes

For more information about amateur radio, visit:
- ARRL: https://www.arrl.org/
- POTA: https://pota.app/
- SOTA: https://www.sota.org.uk/

## Acknowledgments

Special thanks to **KI2D**, author of [Ham2K PoLo](https://polo.ham2k.com/), for creating an excellent portable logging application for amateur radio operators.
