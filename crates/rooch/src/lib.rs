// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::event::EventCommand;
use crate::commands::indexer::Indexer;
use crate::commands::statedb::Statedb;
use cli_types::CommandAction;
use commands::{
    abi::ABI, account::Account, env::Env, genesis::Genesis, init::Init, move_cli::MoveCli,
    object::ObjectCommand, resource::ResourceCommand, rpc::Rpc, server::Server,
    session_key::SessionKey, state::StateCommand, transaction::Transaction,
};
use rooch_types::error::RoochResult;

pub mod cli_types;
pub mod commands;
pub mod utils;

#[derive(clap::Parser)]
#[clap(author, version, about, long_about = None)]
pub struct RoochCli {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(clap::Parser)]
pub enum Command {
    Account(Account),
    Init(Init),
    Move(MoveCli),
    Server(Server),
    State(StateCommand),
    Object(ObjectCommand),
    Resource(ResourceCommand),
    Transaction(Transaction),
    Event(EventCommand),
    ABI(ABI),
    Env(Env),
    SessionKey(SessionKey),
    Rpc(Rpc),
    Statedb(Statedb),
    Indexer(Indexer),
    Genesis(Genesis),
}

pub async fn run_cli(opt: RoochCli) -> RoochResult<String> {
    match opt.cmd {
        Command::Account(account) => account.execute().await,
        Command::Move(move_cli) => move_cli.execute().await,
        Command::Server(server) => server.execute().await,
        Command::Init(init) => init.execute_serialized().await,
        Command::State(state) => state.execute_serialized().await,
        Command::Object(object) => object.execute_serialized().await,
        Command::Resource(resource) => resource.execute_serialized().await,
        Command::Transaction(transation) => transation.execute().await,
        Command::Event(event) => event.execute().await,
        Command::ABI(abi) => abi.execute().await,
        Command::Env(env) => env.execute().await,
        Command::SessionKey(session_key) => session_key.execute().await,
        Command::Rpc(rpc) => rpc.execute().await,
        Command::Statedb(statedb) => statedb.execute().await,
        Command::Indexer(indexer) => indexer.execute().await,
        Command::Genesis(genesis) => genesis.execute().await,
    }
}
