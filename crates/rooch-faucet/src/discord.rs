// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{App, FaucetRequest, FixedBTCAddressRequest, FixedRoochAddressRequest};
use clap::Parser;
use rooch_types::address::{BitcoinAddress, RoochAddress};
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandOptionType};
use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::str::FromStr;
use tokio::spawn;

#[derive(Parser, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub struct DiscordConfig {
    #[arg(long, env = "ROOCH_FAUCET_DISCORD_TOKEN")]
    pub discord_token: Option<String>,
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

        // self.check_gas_balance()

        let discord_handle = spawn(async move {
            loop {
                let channel_id = ChannelId(122);
            }
            // while let Some(message) = faucet_rx.recv().await {
            //     if let FaucetMessage::LowWaterBalance(balance) = message {
            //         // Replace with your channel ID and message content
            //         let channel_id = ChannelId(123456789012345678); // Replace with your actual channel ID
            //         let message_content = format!("Warning: Low water balance: {} units", balance);
            //         if let Err(why) = channel_id.say(&discord.http, &message_content).await {
            //             eprintln!("Error sending message: {:?}", why);
            //         }
            //     }
            // }
        });


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
        println!("Cache built successfully!");

        // It's safe to clone Context, but Arc is cheaper for this use case.
        // Untested claim, just theoretically. :P
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
            // tokio::spawn creates a new green thread that can run in parallel with the rest of
            // the application.
            tokio::spawn(async move {
                loop {
                    log_system_load(&ctx1).await;
                    tokio::time::sleep(Duration::from_secs(120)).await;
                }
            });

            // And of course, we can run more than one thread at different timings.
            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    set_activity_to_current_time(&ctx2);
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}
