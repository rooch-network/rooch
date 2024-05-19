// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::App;
use clap::Parser;
use rooch_types::address::BitcoinAddress;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandOptionType};
use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::str::FromStr;

#[derive(Parser, Clone)]
#[clap(rename_all = "kebab-case")]
pub struct DiscordConfig {
    #[arg(long, env = "ROOCH_FAUCET_DISCORD_TOKEN")]
    pub discord_token: Option<String>,
}

impl App {
    async fn handle_faucet_request(&self, options: &[CommandDataOption]) -> String {
        let value = options
            .get(0)
            .expect("Expected address option")
            .value
            .clone();

        match value {
            CommandDataOptionValue::String(address) => {
                let btc_address =
                    BitcoinAddress::from_str(address.as_str()).expect("Invalid address");

                if let Err(err) = self.request(btc_address).await {
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
                CreateCommandOption::new(CommandOptionType::String, "address", "Your BTC address")
                    .required(true),
            );

        let guild_command = Command::create_global_command(&ctx.http, command).await;
        tracing::info!("I created the following global slash command: {guild_command:#?}");
    }
}
