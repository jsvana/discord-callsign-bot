mod config;
mod github;
mod output;
mod parser;
mod qrz;

use anyhow::Result;
use clap::Parser;
use config::Config;
use github::GitHubClient;
use output::{generate_output_content, OutputEntry};
use parser::CallsignParser;
use qrz::QrzClient;
use serenity::all::GuildId;
use serenity::async_trait;
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Discord bot that generates member lists of amateur radio operators from callsigns
#[derive(Parser, Debug)]
#[command(name = "discord-callsign-bot")]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml", env = "CONFIG_PATH")]
    config: String,
}

struct Handler {
    config: Config,
    parser: CallsignParser,
    qrz_client: Option<Arc<QrzClient>>,
    github_client: GitHubClient,
}

impl Handler {
    fn new(
        config: Config,
        qrz_client: Option<Arc<QrzClient>>,
        github_client: GitHubClient,
    ) -> Self {
        Self {
            config,
            parser: CallsignParser::new(),
            qrz_client,
            github_client,
        }
    }

    async fn generate_member_list(
        &self,
        ctx: &Context,
        guild_config: &config::GuildConfig,
    ) -> Result<()> {
        let guild_id = GuildId::new(guild_config.guild_id);

        info!("Fetching members from guild {}", guild_id);

        // Get all members from the guild
        let members = guild_id
            .members(&ctx.http, None, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch guild members: {}", e))?;

        info!("Found {} members", members.len());

        // Get the bot's own user ID to filter it out
        let bot_user_id = ctx.cache.current_user().id;

        let mut entries = Vec::new();

        for member in members {
            // Skip the bot itself
            if member.user.id == bot_user_id {
                info!("Skipping bot user: {}", member.user.name);
                continue;
            }

            // Try to find a valid callsign in multiple name fields
            // Priority: nick -> global_name -> user.name
            let name_fields = [
                member.nick.as_ref(),
                member.user.global_name.as_ref(),
                Some(&member.user.name),
            ];

            let (parsed, display_name) = name_fields
                .iter()
                .filter_map(|field| {
                    field.map(|name| {
                        let parsed = self.parser.parse(name);
                        (parsed, name.clone())
                    })
                })
                .find(|(parsed, _)| parsed.is_some())
                .unwrap_or((None, member.user.name.clone()));

            info!(
                "Processing member: {} (parsed: {})",
                display_name,
                if parsed.is_some() { "✓" } else { "✗" }
            );

            // Check if there's a manual override for this user
            let user_id = member.user.id.to_string();
            if let Some(override_config) = guild_config.get_override(&user_id) {
                info!("Using override for user {}", user_id);

                // Use the parsed callsign if available

                let callsign = override_config
                    .callsign
                    .clone()
                    .or_else(|| parsed.as_ref().map(|p| p.callsign.clone()))
                    .unwrap_or_else(|| "UNKNOWN".to_string());

                let name = override_config
                    .name
                    .clone()
                    .or_else(|| parsed.as_ref().map(|p| p.name.clone()))
                    .unwrap_or_else(|| display_name.clone());

                let suffix = override_config
                    .suffix
                    .clone()
                    .unwrap_or_else(|| guild_config.output.default_suffix.clone());

                let emoji_separator = override_config
                    .emoji
                    .clone()
                    .unwrap_or_else(|| guild_config.output.emoji_separator.clone());

                entries.push(OutputEntry {
                    callsign,
                    name,
                    suffix,
                    emoji_separator,
                });
            } else if let Some(parsed) = parsed {
                // Successfully parsed callsign from one of the name fields
                let mut name = parsed.name.clone();

                // Try to get name from QRZ if client is available
                if let Some(qrz_client) = &self.qrz_client {
                    match qrz_client.lookup_callsign(&parsed.callsign).await {
                        Ok(qrz_info) => {
                            if let Some(qrz_name) = QrzClient::get_display_name(&qrz_info) {
                                info!(
                                    "Using QRZ name '{}' for callsign {}",
                                    qrz_name, parsed.callsign
                                );
                                name = qrz_name;
                            } else {
                                info!(
                                    "No name found in QRZ for {}, using Discord name: {}",
                                    parsed.callsign, name
                                );
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to lookup callsign {} in QRZ: {:?}. Using Discord name: {}",
                                parsed.callsign, e, name
                            );
                        }
                    }
                }

                entries.push(OutputEntry {
                    callsign: parsed.callsign,
                    name,
                    suffix: guild_config.output.default_suffix.clone(),
                    emoji_separator: guild_config.output.emoji_separator.clone(),
                });
            } else {
                info!(
                    "Could not parse callsign from display name: {}",
                    display_name
                );
            }
        }

        // Deduplicate entries by callsign (keep first occurrence)
        let mut seen_callsigns = HashMap::new();
        let mut unique_entries = Vec::new();

        for entry in entries {
            if !seen_callsigns.contains_key(&entry.callsign) {
                seen_callsigns.insert(entry.callsign.clone(), true);
                unique_entries.push(entry);
            } else {
                warn!(
                    "Skipping duplicate callsign: {} (already processed)",
                    entry.callsign
                );
            }
        }

        info!(
            "Committing {} unique entries to GitHub (filtered {} duplicates)",
            unique_entries.len(),
            seen_callsigns.len() - unique_entries.len()
        );

        // Generate content and commit to GitHub
        let content = generate_output_content(unique_entries, guild_config.output.title.as_deref());

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

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        info!("{} is connected and ready!", ready.user.name);

        // Process each configured guild
        for guild_config in &self.config.guilds {
            let guild_id = GuildId::new(guild_config.guild_id);
            info!("Processing guild: {}", guild_id);

            // Set bot nickname if configured for this guild
            if let Some(nickname) = &guild_config.bot_nickname {
                if let Err(e) = guild_id.edit_nickname(&ctx.http, Some(nickname)).await {
                    warn!(
                        "Failed to set bot nickname to '{}' in guild {}: {}",
                        nickname, guild_id, e
                    );
                } else {
                    info!("Set bot nickname to '{}' in guild {}", nickname, guild_id);
                }
            }

            // Generate the member list when the bot starts
            if let Err(e) = self.generate_member_list(&ctx, guild_config).await {
                error!(
                    "Failed to generate member list for guild {}: {:?}",
                    guild_id, e
                );
                std::process::exit(1);
            }
        }

        info!("Member list generation complete for all guilds. Bot is now listening for member changes.");
    }

    async fn guild_member_addition(
        &self,
        ctx: Context,
        new_member: serenity::model::guild::Member,
    ) {
        let guild_id = new_member.guild_id.get();

        // Check if this guild is configured
        if let Some(guild_config) = self.config.get_guild_config(guild_id) {
            info!(
                "New member joined guild {}: {}",
                guild_id, new_member.user.name
            );

            if let Err(e) = self.generate_member_list(&ctx, guild_config).await {
                error!(
                    "Failed to regenerate member list for guild {} after member addition: {:?}",
                    guild_id, e
                );
            } else {
                info!(
                    "Member list updated for guild {} after new member joined",
                    guild_id
                );
            }
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: serenity::model::user::User,
        _member_data_if_available: Option<serenity::model::guild::Member>,
    ) {
        let guild_id_u64 = guild_id.get();

        // Check if this guild is configured
        if let Some(guild_config) = self.config.get_guild_config(guild_id_u64) {
            info!("Member left guild {}: {}", guild_id_u64, user.name);

            if let Err(e) = self.generate_member_list(&ctx, guild_config).await {
                error!(
                    "Failed to regenerate member list for guild {} after member removal: {:?}",
                    guild_id_u64, e
                );
            } else {
                info!(
                    "Member list updated for guild {} after member left",
                    guild_id_u64
                );
            }
        }
    }

    async fn guild_member_update(
        &self,
        ctx: Context,
        _old_if_available: Option<serenity::model::guild::Member>,
        new: Option<serenity::model::guild::Member>,
        event: serenity::model::event::GuildMemberUpdateEvent,
    ) {
        let guild_id = event.guild_id.get();

        // Check if this guild is configured
        if let Some(guild_config) = self.config.get_guild_config(guild_id) {
            if let Some(member) = new {
                info!("Member updated in guild {}: {}", guild_id, member.user.name);

                if let Err(e) = self.generate_member_list(&ctx, guild_config).await {
                    error!(
                        "Failed to regenerate member list for guild {} after member update: {:?}",
                        guild_id, e
                    );
                } else {
                    info!(
                        "Member list updated for guild {} after member info changed",
                        guild_id
                    );
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("qrz_xml=off".parse().unwrap()),
        )
        .init();

    // Load configuration
    let config = Config::from_file(&args.config)?;

    info!("Configuration loaded from: {}", args.config);

    // Initialize QRZ client if credentials are configured
    let qrz_client = if let Some(qrz_config) = &config.qrz {
        info!("QRZ credentials found, initializing QRZ client...");
        match QrzClient::new(qrz_config).await {
            Ok(client) => {
                info!("QRZ client initialized successfully");
                Some(Arc::new(client))
            }
            Err(e) => {
                warn!(
                    "Failed to initialize QRZ client: {:?}. Continuing without QRZ lookups.",
                    e
                );
                None
            }
        }
    } else {
        info!("No QRZ credentials configured, skipping QRZ lookups");
        None
    };

    // Initialize GitHub client
    info!("Initializing GitHub client...");
    let github_client = GitHubClient::new()?;
    info!("GitHub client initialized successfully");

    // Set up Discord client
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(&config.discord.token, intents)
        .event_handler(Handler::new(config, qrz_client, github_client))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create Discord client: {}", e))?;

    // Start the bot
    info!("Starting Discord bot...");
    client
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start Discord client: {}", e))?;

    Ok(())
}
