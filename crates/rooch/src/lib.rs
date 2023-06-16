// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::event::EventCommand;
use cli_types::CommandAction;
use commands::{
    account::Account, init::Init, move_cli::MoveCli, object::ObjectCommand,
    resource::ResourceCommand, server::Server, state::StateCommand, transaction::Transaction,
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
    }
}
