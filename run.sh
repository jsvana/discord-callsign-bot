#!/bin/bash
# Simple runner script for the Discord callsign bot

set -e

CONFIG_FILE="${CONFIG_PATH:-config.toml}"

# Check if config exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Error: Configuration file not found: $CONFIG_FILE"
    echo "Please copy config.toml.example to config.toml and configure it."
    exit 1
fi

# Build and run
echo "Building discord-callsign-bot..."
cargo build --release

echo "Running bot with config: $CONFIG_FILE"
CONFIG_PATH="$CONFIG_FILE" ./target/release/discord-callsign-bot

echo "Bot execution complete!"
