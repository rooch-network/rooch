// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{App, FaucetRequest, FixedBTCAddressRequest, FixedRoochAddressRequest};
use clap::Parser;
use rooch_types::address::{BitcoinAddress, RoochAddress};
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandOptionType};
use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage,
};
use serenity::model::{
    application::{Command, Interaction},
    gateway::Ready,
    id::{ChannelId, GuildId},
};
use serenity::prelude::*;
use std::{
    str::FromStr,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

#[derive(Parser, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub struct DiscordConfig {
    #[arg(long, env = "ROOCH_FAUCET_DISCORD_TOKEN")]
    pub discord_token: Option<String>,

    #[arg(
        long,
        env = "ROOCH_FAUCET_NOTIFY_CHANNEL_ID",
        default_value = "0"
    )]
    pub notify_channel_id: u64,

    #[arg(long, env = "ROOCH_FAUCET_CHECK_INTERVAL", default_value = "3600")]
    pub check_interval: u64,

    #[arg(long, env = "ROOCH_FAUCET_NOTIFY_THRESHOLD", default_value = "1000")]
    pub notify_threshold: u64,
}

impl App {
    async fn handle_faucet_request(&self, options: &[CommandDataOption]) -> String {
        let value = options
            .first()
            .expect("Expected address option")
            .value
            .clone();

        match value {
            CommandDataOptionValue::String(address) => {
                let request = match address.starts_with("0x") {
                    true => FaucetRequest::FixedRoochAddressRequest(FixedRoochAddressRequest {
                        recipient: RoochAddress::from_str(address.as_str())
                            .expect("Invalid address"),
                    }),
                    false => FaucetRequest::FixedBTCAddressRequest(FixedBTCAddressRequest {
                        recipient: BitcoinAddress::from_str(address.as_str())
                            .expect("Invalid address"),
                    }),
                };

                if let Err(err) = self.request(request).await {
                    tracing::error!("Failed make faucet request for {address:?}: {}", err);
                    format!("Internal Error: Failed to send funds to {address:?}")
                } else {
                    format!("Sending funds to {address:?}")
                }
            }
            _ => "No address found!".to_string(),
        }
    }
}

#[async_trait]
impl EventHandler for App {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            tracing::info!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "faucet" => self.handle_faucet_request(&command.data.options).await,
                _ => "not implemented".to_string(),
            };

            let data = CreateInteractionResponseMessage::new().content(content);
            let builder = CreateInteractionResponse::Message(data);
            if let Err(why) = command.create_response(&ctx.http, builder).await {
                tracing::error!("Cannot respond to slash command: {:#?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);

        let command = CreateCommand::new("faucet")
            .description("Request funds from the faucet")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "address",
                    "Your BTC/Rooch address",
                )
                .required(true),
            );

        let guild_command = Command::create_global_command(&ctx.http, command).await;
        tracing::info!("I created the following global slash command: {guild_command:#?}");
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        tracing::info!("Cache built successfully!");

        let discord_cfg = self.discord_config.clone();

        if discord_cfg.notify_channel_id == 0 {
            tracing::info!("Notify channel id is zero, not check gas balance!");
            return;
        }

        let ctx = Arc::new(ctx);

        // We need to check that the loop is not already running when this event triggers, as this
        // event triggers every time the bot enters or leaves a guild, along every time the ready
        // shard event triggers.
        //
        // An AtomicBool is used because it doesn't require a mutable reference to be changed, as
        // we don't have one due to self being an immutable reference.
        if !self.is_loop_running.load(Ordering::Relaxed) {
            // We have to clone the Arc, as it gets moved into the new thread.
            let ctx1 = Arc::clone(&ctx);
            let app = Arc::new(self.clone());
            // tokio::spawn creates a new green thread that can run in parallel with the rest of
            // the application.
            tokio::spawn(async move {
                loop {
                    let result = app.check_gas_balance().await;

                    match result {
                        Ok(v) => {
                            if v < discord_cfg.notify_threshold as f64 {
                                let embed = CreateEmbed::new()
                                    .title("Insufficient gas balance")
                                    .field("current balance", v.to_string(), true);
                                let builder = CreateMessage::new().embed(embed);
                                let message = ChannelId::new(discord_cfg.notify_channel_id)
                                    .send_message(&ctx1, builder)
                                    .await;
                                if let Err(why) = message {
                                    tracing::error!("Error sending message: {why:?}");
                                };
                            }
                        }
                        Err(e) => {
                            let embed = CreateEmbed::new().title("Check gas balance failed").field(
                                "error",
                                e.to_string(),
                                false,
                            );
                            let builder = CreateMessage::new().embed(embed);
                            let message = ChannelId::new(discord_cfg.notify_channel_id)
                                .send_message(&ctx1, builder)
                                .await;
                            if let Err(why) = message {
                                tracing::error!("Error sending message: {why:?}");
                            };
                        }
                    }

                    tokio::time::sleep(Duration::from_secs(discord_cfg.check_interval)).await;
                }
            });

            let ctx2 = Arc::clone(&ctx);
            let app2 = Arc::new(self.clone());
            tokio::spawn(async move {
                while let Some(err) = app2.err_receiver.write().await.recv().await {
                    let embed = CreateEmbed::new().title("Sending gas funds failed").field(
                        "error",
                        err.to_string(),
                        false,
                    );
                    let builder = CreateMessage::new().embed(embed);
                    let message = ChannelId::new(discord_cfg.notify_channel_id)
                        .send_message(&ctx2, builder)
                        .await;
                    if let Err(why) = message {
                        tracing::error!("Error sending message: {why:?}");
                    };
                }
            });

            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}
