use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub output: OutputConfig,
    #[serde(default)]
    pub overrides: HashMap<String, Override>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordConfig {
    pub token: String,
    pub guild_id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    pub file_path: String,
    pub default_suffix: String,
    #[serde(default = "default_emoji_separator")]
    pub emoji_separator: String,
}

fn default_emoji_separator() -> String {
    "📻".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Override {
    pub callsign: Option<String>,
    pub name: Option<String>,
    pub suffix: Option<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    pub fn get_override(&self, discord_id: &str) -> Option<&Override> {
        self.overrides.get(discord_id)
    }
}
