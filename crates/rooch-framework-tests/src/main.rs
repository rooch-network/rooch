// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use coerce::actor::{system::ActorSystem, IntoActor};
use rooch_framework_tests::bitcoin_block_tester::TesterGenesisBuilder;
use rooch_relayer::actor::{
    bitcoin_client::BitcoinClientActor, bitcoin_client_proxy::BitcoinClientProxy,
};

#[derive(Parser)]
#[clap(name = "test_builder", author = "The Rooch Core Contributors")]
struct TestBuilderOpts {
    #[clap(
        long,
        env = "BITCOIN_RPC_URL",
        requires = "btc-rpc-username",
        requires = "btc-rpc-password"
    )]
    pub btc_rpc_url: String,

    #[clap(long, id = "btc-rpc-username", env = "BTC_RPC_USERNAME")]
    pub btc_rpc_username: String,

    #[clap(long, id = "btc-rpc-password", env = "BTC_RPC_PASSWORD")]
    pub btc_rpc_password: String,

    #[clap(long, id = "ord-events-dir")]
    pub ord_events_dir: PathBuf,

    /// Block heights to execute
    #[clap(long, id = "blocks")]
    pub blocks: Vec<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let opts: TestBuilderOpts = TestBuilderOpts::parse();
    let actor_system = ActorSystem::global_system();
    let bitcoin_client = BitcoinClientActor::new(
        &opts.btc_rpc_url,
        &opts.btc_rpc_username,
        &opts.btc_rpc_password,
    )?;
    let bitcoin_client_actor_ref = bitcoin_client
        .into_actor(Some("bitcoin_client_for_rpc_service"), &actor_system)
        .await?;
    let bitcoin_client_proxy = BitcoinClientProxy::new(bitcoin_client_actor_ref.into());
    let mut builder = TesterGenesisBuilder::new(bitcoin_client_proxy, opts.ord_events_dir)?;
    let mut blocks = opts.blocks;
    blocks.sort();
    for block in blocks {
        builder = builder.add_block(block).await?;
    }
    let genesis = builder.build().await?;
    genesis.save()?;
    Ok(())
}
