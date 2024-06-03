// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use prometheus::Registry;
use rooch_faucet::{serve, App, DiscordConfig, Faucet, FaucetConfig, WebConfig};
use serenity::prelude::*;
use tokio::{
    spawn,
    sync::mpsc::{self},
};
use tracing::warn;

#[derive(Parser, Clone)]
#[clap(
    name = "Rooch Faucet",
    about = "Faucet for requesting test tokens on Rooch",
    rename_all = "kebab-case"
)]
pub struct Config {
    #[clap(flatten)]
    pub web_config: WebConfig,

    #[clap(flatten)]
    pub faucet_config: FaucetConfig,

    #[clap(flatten)]
    pub discord_config: DiscordConfig,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = Config::parse();

    let Config {
        web_config,
        faucet_config,
        discord_config,
        ..
    } = config;

    let registry = Registry::new();
    let (sender, receiver) = mpsc::channel(faucet_config.max_request_queue_length as usize);
    let (err_sender, err_receiver) = mpsc::channel(faucet_config.max_request_queue_length as usize);
    let app = App::new(
        sender,
        faucet_config.wallet_config_dir.clone(),
        discord_config.clone(),
        err_receiver,
        faucet_config.faucet_grant_amount,
    );
    let faucet = Faucet::new(&registry, faucet_config, receiver, err_sender)
        .await
        .expect("Failed to create faucet");

    let discord_client = if let Some(token) = discord_config
        .discord_token
        .clone()
        .filter(|token| !token.is_empty())
    {
        // Set gateway intents, which decides what events the bot will be notified about
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILDS
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        // Create a new instance of the Client, logging in as a bot.
        let client = Client::builder(token, intents)
            .event_handler(app.clone())
            .await
            .expect("Err creating client");

        Some(client)
    } else {
        warn!("Discord bot disabled. For local testing this is fine.");
        None
    };

    let faucet_handle = spawn(faucet.start());
    let api_handle = spawn(serve(app, web_config));

    if let Some(mut discord) = discord_client {
        let _result = futures::join!(faucet_handle, api_handle, discord.start());
    } else {
        let _result = futures::join!(faucet_handle, api_handle);
    };

    Ok(())
}
