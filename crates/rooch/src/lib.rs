// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::DB;
use crate::commands::event::EventCommand;
use crate::commands::indexer::Indexer;
use crate::commands::statedb::Statedb;
use clap::builder::{
    styling::{AnsiColor, Effects},
    Styles,
};
use cli_types::CommandAction;
use commands::{
    abi::ABI, account::Account, dynamic_field::DynamicField, env::Env, genesis::Genesis,
    init::Init, move_cli::MoveCli, object::ObjectCommand, resource::ResourceCommand, rpc::Rpc,
    server::Server, session_key::SessionKey, state::StateCommand, transaction::Transaction,
    upgrade::Upgrade, util::Util, version::Version,
};
use once_cell::sync::Lazy;
use rooch_types::error::RoochResult;

pub mod cli_types;
pub mod commands;
pub mod utils;

#[derive(clap::Parser)]
#[clap(author, long_version = LONG_VERSION.as_str(), about, long_about = None,
styles = Styles::styled()
.header(AnsiColor::Green.on_default() | Effects::BOLD)
.usage(AnsiColor::Green.on_default() | Effects::BOLD)
.literal(AnsiColor::Blue.on_default() | Effects::BOLD)
.placeholder(AnsiColor::Cyan.on_default()))]
pub struct RoochCli {
    #[clap(subcommand)]
    pub cmd: Command,
}

static LONG_VERSION: Lazy<String> = Lazy::new(|| {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let git_commit_hash = env!("VERGEN_GIT_SHA");
    format!("{} (git commit {})", cargo_version, git_commit_hash)
});

#[allow(clippy::large_enum_variant)]
#[derive(clap::Parser)]
pub enum Command {
    Version(Version),
    Account(Account),
    Init(Init),
    Move(MoveCli),
    Server(Server),
    State(StateCommand),
    Object(ObjectCommand),
    DynamicField(DynamicField),
    Resource(ResourceCommand),
    #[clap(visible_alias = "tx")]
    Transaction(Transaction),
    Event(EventCommand),
    ABI(ABI),
    Env(Env),
    SessionKey(SessionKey),
    Rpc(Rpc),
    Statedb(Statedb),
    Indexer(Indexer),
    Genesis(Genesis),
    Upgrade(Upgrade),
    DB(DB),
    Util(Util),
}

pub async fn run_cli(opt: RoochCli) -> RoochResult<String> {
    match opt.cmd {
        Command::Version(version) => version.execute().await,
        Command::Account(account) => account.execute().await,
        Command::Move(move_cli) => move_cli.execute().await,
        Command::Server(server) => server.execute().await,
        Command::Init(init) => init.execute_serialized().await,
        Command::State(state) => state.execute_serialized().await,
        Command::Object(object) => object.execute_serialized().await,
        Command::DynamicField(dynamic_field) => dynamic_field.execute().await,
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
        Command::Upgrade(upgrade) => upgrade.execute().await,
        Command::DB(db) => db.execute().await,
        Command::Util(util) => util.execute().await,
    }
}
