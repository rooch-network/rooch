// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use commands::{
    account::Account, init::Init, move_cli::MoveCli, server::Server, state::StateCommand,
};
use rooch_types::error::RoochResult;
use types::CommandAction;

pub mod commands;
pub mod types;
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
}

pub async fn run_cli(opt: RoochCli) -> RoochResult<String> {
    match opt.cmd {
        Command::Move(move_cli) => move_cli.execute().await,
        Command::Server(server) => server.execute().await,
        Command::Init(c) => c.execute_serialized().await,
        Command::State(s) => s.execute_serialized().await,
        Command::Account(a) => a.execute().await,
    }
}
