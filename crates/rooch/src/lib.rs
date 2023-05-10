// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::account::AccountCommand;
use crate::commands::{object::ObjectCommand, resource::ResourceCommand};
use anyhow::Result;

pub mod commands;
pub mod config;

#[derive(clap::Parser)]
#[clap(author, version, about, long_about = None)]
pub struct RoochCli {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(clap::Parser)]
pub enum Command {
    Move(moveos_cli::MoveCli),
    Server(moveos_server::OsServer),
    Resource(ResourceCommand),
    Object(ObjectCommand),
    #[clap(subcommand)]
    Account(AccountCommand),
}

pub async fn run_cli(opt: RoochCli) -> Result<()> {
    match opt.cmd {
        Command::Move(move_cli) => moveos_cli::run_cli(move_cli).await,
        Command::Server(os) => os.execute().await,
        Command::Resource(resource) => resource.execute().await,
        Command::Object(object) => object.execute().await,
        Command::Account(account) => account.execute().await,
    }
}
