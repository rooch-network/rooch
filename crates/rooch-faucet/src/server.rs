// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::faucet_proxy::FaucetProxy;
use crate::*;
use coerce::actor::{system::ActorSystem, IntoActor};
use prometheus::Registry;
use rooch_rpc_client::wallet_context::WalletContext;
use serenity::{all::GatewayIntents, Client};
use tokio::{
    spawn,
    sync::mpsc::{self},
};
use tracing::warn;

pub async fn start(
    wallet_context: WalletContext,
    web_config: WebConfig,
    faucet_config: FaucetConfig,
    discord_config: DiscordConfig,
) -> anyhow::Result<String> {
    let registry = Registry::new();
    let actor_system = ActorSystem::global_system();

    let (err_sender, err_receiver) = mpsc::channel(faucet_config.max_request_queue_length as usize);

    let faucet = Faucet::new(&registry, wallet_context, faucet_config, err_sender)?;

    let faucet_actor_ref = faucet
        .into_actor(Some("FaucetActor"), &actor_system)
        .await?;
    let faucet_proxy = FaucetProxy::new(faucet_actor_ref.into());

    let app = App::new(faucet_proxy, err_receiver, discord_config.clone());

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

    let api_handle = spawn(serve(app, web_config));

    if let Some(mut discord) = discord_client {
        let _result = futures::join!(api_handle, discord.start());
    } else {
        let _result = futures::join!(api_handle);
    };
    Ok("Faucet server stopped".to_string())
}
