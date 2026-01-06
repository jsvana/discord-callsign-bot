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

## Step 2: Get a GitHub Token

1. Visit https://github.com/settings/tokens
2. Click "Generate new token" (classic)
3. Select `repo` scope
4. Click "Generate token" and copy it (save it somewhere safe)

## Step 3: Invite Bot to Your Server

1. In the Discord Developer Portal, go to "OAuth2" → "URL Generator"
2. Select scopes:
   - ✅ `bot`
3. Select permissions:
   - ✅ `Read Messages/View Channels`
4. Copy the generated URL and open it in your browser
5. Select your server and authorize

## Step 4: Get Your Server ID

1. In Discord: Settings → Advanced → Enable "Developer Mode"
2. Right-click your server icon → "Copy Server ID"

## Step 5: Configure

```bash
cd discord-callsign-bot
cp config.toml.example config.toml
nano config.toml  # or use your favorite editor
```

Update these values:
```toml
[discord]
token = "YOUR_BOT_TOKEN_FROM_STEP_1"

[[guilds]]
guild_id = YOUR_SERVER_ID_FROM_STEP_4

[guilds.output]
repo = "your-username/your-repo"
path = "members.txt"
branch = "main"
default_suffix = "73"
```

Save and exit.

## Step 6: Run

```bash
# Option 1: Using the run script (set GITHUB_TOKEN first)
GITHUB_TOKEN=your_token ./run.sh

# Option 2: Using make
GITHUB_TOKEN=your_token make run

# Option 3: Using cargo directly
GITHUB_TOKEN=your_token cargo run --release
```

## Step 7: Check Output

Check your GitHub repository for the committed `members.txt` file.

You should see content like:
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

**GitHub commit errors?**
- Make sure `GITHUB_TOKEN` is set and has `repo` scope
- Verify the repository exists and you have write access

## Adding Overrides

If someone's callsign isn't parsing correctly:

1. Right-click their name in Discord → "Copy User ID"
2. Add to `config.toml` under the appropriate guild:

```toml
[guilds.overrides."THEIR_USER_ID"]
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
0 0 * * * cd /path/to/discord-callsign-bot && GITHUB_TOKEN=your_token ./run.sh
```

**Run on demand:**
```bash
GITHUB_TOKEN=your_token ./run.sh
```

**Docker deployment:**
```bash
GITHUB_TOKEN=your_token make docker-run
```

Need help? Check the [README.md](README.md) or open an issue on GitHub.
