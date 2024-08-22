// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bitcoin::BlockHash;
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

    #[clap(long, id = "block-hash", env = "BLOCK_HASH")]
    pub block_hash: BlockHash,
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
    let builder = TesterGenesisBuilder::new(bitcoin_client_proxy)?;
    let builder = builder.add_block(opts.block_hash).await?;
    let genesis = builder.build().await?;
    genesis.save()?;
    Ok(())
}
