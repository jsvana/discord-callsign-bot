# Quick Start Guide

Get your Discord callsign bot up and running in 5 minutes.

## Step 1: Get a Discord Bot Token

1. Visit https://discord.com/developers/applications
2. Click "New Application", name it (e.g., "Callsign Bot")
3. Go to "Bot" → "Add Bot"
4. Enable these intents under "Privileged Gateway Intents":
   - ✅ SERVER MEMBERS INTENT (required!)
   - ✅ PRESENCE INTENT
5. Click "Reset Token" and copy the token (save it somewhere safe)

## Step 2: Invite Bot to Your Server

1. In the Discord Developer Portal, go to "OAuth2" → "URL Generator"
2. Select scopes:
   - ✅ `bot`
3. Select permissions:
   - ✅ `Read Messages/View Channels`
4. Copy the generated URL and open it in your browser
5. Select your server and authorize

## Step 3: Get Your Server ID

1. In Discord: Settings → Advanced → Enable "Developer Mode"
2. Right-click your server icon → "Copy Server ID"

## Step 4: Configure

```bash
cd discord-callsign-bot
cp config.toml.example config.toml
nano config.toml  # or use your favorite editor
```

Update these values:
```toml
[discord]
token = "YOUR_BOT_TOKEN_FROM_STEP_1"
guild_id = YOUR_SERVER_ID_FROM_STEP_3

[output]
file_path = "members.txt"
default_suffix = "73"
```

Save and exit.

## Step 5: Run

```bash
# Option 1: Using the run script
./run.sh

# Option 2: Using make
make run

# Option 3: Using cargo directly
cargo run --release
```

## Step 6: Check Output

```bash
cat members.txt
```

You should see output like:
```
KI7QCF Forrest 73
W6JSV Jay 73
```

## Troubleshooting

**Bot can't see members?**
- Make sure "SERVER MEMBERS INTENT" is enabled (Step 1)
- Restart the bot after enabling intents

**No callsigns found?**
- Check that Discord member names contain callsigns like "W6JSV - Jay" or "Forrest KI7QCF"

**Build errors?**
- Make sure you have Rust installed: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Adding Overrides

If someone's callsign isn't parsing correctly:

1. Right-click their name in Discord → "Copy User ID"
2. Add to `config.toml`:

```toml
[overrides."THEIR_USER_ID"]
callsign = "W1ABC"
name = "John"
suffix = "CQ"
```

## Next Steps

- Read the full [README.md](README.md) for advanced configuration
- Set up as a systemd service for automatic runs
- Schedule with cron for periodic updates

## Common Use Cases

**Daily member list update:**
```bash
# Add to crontab (crontab -e)
0 0 * * * cd /path/to/discord-callsign-bot && ./run.sh
```

**Run on demand:**
```bash
./run.sh && cat members.txt
```

**Docker deployment:**
```bash
make docker-run
```

Need help? Check the [README.md](README.md) or open an issue on GitHub.
