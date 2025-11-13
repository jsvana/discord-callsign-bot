# Discord Callsign Bot

A Rust-based Discord bot that reads server members with amateur radio callsigns in their names and generates a formatted text file.

## Features

- **Automatic Callsign Parsing**: Extracts callsigns from Discord display names in various formats:
  - `W6JSV - Jay` → Callsign: W6JSV, Name: Jay
  - `Forrest KI7QCF` → Callsign: KI7QCF, Name: Forrest
  - `Jay (W6JSV)` → Callsign: W6JSV, Name: Jay

- **Manual Overrides**: Configure specific callsigns, names, or suffix text for individual users via TOML configuration

- **Configurable Output**: Customize the output format and default suffix text

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

### 2. Get Your Guild (Server) ID

1. In Discord, go to User Settings → Advanced → Enable "Developer Mode"
2. Right-click on your server icon and click "Copy Server ID"

### 3. Configure the Bot

```bash
# Copy the example configuration
cp config.toml.example config.toml

# Edit the configuration file
nano config.toml
```

Update the following fields:
- `discord.token`: Your bot token from step 1
- `discord.guild_id`: Your server ID from step 2
- `output.file_path`: Where to save the output file (default: `members.txt`)
- `output.default_suffix`: Text to append after each entry (e.g., "73")

### 4. Add Manual Overrides (Optional)

To override callsign/name/suffix for specific users:

1. In Discord, right-click on a user and select "Copy User ID"
2. Add an override section in `config.toml`:

```toml
[overrides."USER_ID_HERE"]
callsign = "W1ABC"  # Optional: override callsign
name = "John"       # Optional: override name
suffix = "CQ CQ"    # Optional: override suffix text
```

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
3. Parse callsigns and names
4. Generate the output file
5. Automatically shut down

### Output Format

The generated file will have one line per member:
```
<CALLSIGN> <NAME> <SUFFIX>
```

Example output (`members.txt`):
```
KI7QCF Forrest 73
W6JSV Jay 73
W1ABC John CQ CQ
```

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
- `guild_id` (required): The Discord server ID to read members from

### `[output]`
- `file_path` (required): Path where the member list will be saved
- `default_suffix` (required): Default text appended after each member entry

### `[overrides."USER_ID"]`
All fields are optional. Only specify what you want to override:
- `callsign`: Override the parsed callsign
- `name`: Override the parsed name
- `suffix`: Override the default suffix for this user

## Troubleshooting

### Bot can't see members
- Ensure "SERVER MEMBERS INTENT" is enabled in the Discord Developer Portal
- Make sure the bot has permission to view members in your server

### No callsigns found
- Check that member display names contain valid amateur radio callsigns
- Valid formats: W6JSV, KI7QCF, N0CALL, etc.
- Callsigns must follow the pattern: `[PREFIX][DIGIT][SUFFIX]`

### Permission denied errors
- Ensure the output directory is writable
- Check file system permissions for the configured `output.file_path`

## License

MIT

## Contributing

Pull requests welcome! Please ensure code is formatted with `cargo fmt` and passes `cargo clippy` before submitting.

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
