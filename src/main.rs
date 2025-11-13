mod config;
mod output;
mod parser;
mod qrz;

use anyhow::Result;
use config::Config;
use output::{write_output_file, OutputEntry};
use parser::CallsignParser;
use qrz::QrzClient;
use serenity::all::GuildId;
use serenity::async_trait;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;
use tracing::{error, info, warn};

struct Handler {
    config: Config,
    parser: CallsignParser,
    qrz_client: Option<Arc<QrzClient>>,
}

impl Handler {
    fn new(config: Config, qrz_client: Option<Arc<QrzClient>>) -> Self {
        Self {
            config,
            parser: CallsignParser::new(),
            qrz_client,
        }
    }

    async fn generate_member_list(&self, ctx: &Context) -> Result<()> {
        let guild_id = GuildId::new(self.config.discord.guild_id);

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

            // Get the display name (nickname if set, otherwise username)
            let display_name = member
                .nick
                .as_ref()
                .unwrap_or(&member.user.name)
                .to_string();

            info!("Processing member: {}", display_name);

            // Check if there's a manual override for this user
            let user_id = member.user.id.to_string();
            if let Some(override_config) = self.config.get_override(&user_id) {
                info!("Using override for user {}", user_id);

                // Parse normally first to get defaults
                let parsed = self.parser.parse(&display_name);

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
                    .unwrap_or_else(|| self.config.output.default_suffix.clone());

                entries.push(OutputEntry {
                    callsign,
                    name,
                    suffix,
                });
            } else if let Some(parsed) = self.parser.parse(&display_name) {
                // Successfully parsed callsign from display name
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
                    suffix: self.config.output.default_suffix.clone(),
                });
            } else {
                info!(
                    "Could not parse callsign from display name: {}",
                    display_name
                );
            }
        }

        info!("Writing {} entries to file", entries.len());

        // Write the output file
        write_output_file(
            &self.config.output.file_path,
            entries,
            &self.config.output.emoji_separator,
            self.config.output.title.as_deref(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to write output file: {}", e))?;

        info!(
            "Successfully generated member list at: {}",
            self.config.output.file_path
        );

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        info!("{} is connected and ready!", ready.user.name);

        // Set bot nickname if configured
        if let Some(nickname) = &self.config.discord.bot_nickname {
            let guild_id = GuildId::new(self.config.discord.guild_id);
            if let Err(e) = guild_id.edit_nickname(&ctx.http, Some(nickname)).await {
                warn!("Failed to set bot nickname to '{}': {}", nickname, e);
            } else {
                info!("Set bot nickname to: {}", nickname);
            }
        }

        // Generate the member list when the bot starts
        if let Err(e) = self.generate_member_list(&ctx).await {
            error!("Failed to generate member list: {:?}", e);
            std::process::exit(1);
        }

        info!("Member list generation complete. Bot is now listening for member changes.");
    }

    async fn guild_member_addition(
        &self,
        ctx: Context,
        new_member: serenity::model::guild::Member,
    ) {
        info!("New member joined: {}", new_member.user.name);

        if let Err(e) = self.generate_member_list(&ctx).await {
            error!(
                "Failed to regenerate member list after member addition: {:?}",
                e
            );
        } else {
            info!("Member list updated after new member joined");
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild_id: GuildId,
        user: serenity::model::user::User,
        _member_data_if_available: Option<serenity::model::guild::Member>,
    ) {
        info!("Member left: {}", user.name);

        if let Err(e) = self.generate_member_list(&ctx).await {
            error!(
                "Failed to regenerate member list after member removal: {:?}",
                e
            );
        } else {
            info!("Member list updated after member left");
        }
    }

    async fn guild_member_update(
        &self,
        ctx: Context,
        _old_if_available: Option<serenity::model::guild::Member>,
        new: Option<serenity::model::guild::Member>,
        _event: serenity::model::event::GuildMemberUpdateEvent,
    ) {
        if let Some(member) = new {
            info!("Member updated: {}", member.user.name);

            if let Err(e) = self.generate_member_list(&ctx).await {
                error!(
                    "Failed to regenerate member list after member update: {:?}",
                    e
                );
            } else {
                info!("Member list updated after member info changed");
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load configuration
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config = Config::from_file(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    info!("Configuration loaded from: {}", config_path);

    // Initialize QRZ client if credentials are configured
    let qrz_client = if let Some(qrz_config) = &config.qrz {
        info!("QRZ credentials found, initializing QRZ client...");
        match QrzClient::new(qrz_config).await {
            Ok(client) => {
                info!("QRZ client initialized successfully");
                Some(Arc::new(client))
            }
            Err(e) => {
                warn!("Failed to initialize QRZ client: {:?}. Continuing without QRZ lookups.", e);
                None
            }
        }
    } else {
        info!("No QRZ credentials configured, skipping QRZ lookups");
        None
    };

    // Set up Discord client
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(&config.discord.token, intents)
        .event_handler(Handler::new(config, qrz_client))
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
