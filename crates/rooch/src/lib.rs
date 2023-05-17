// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::{
    account::Account,
    move_cli::{self, MoveCli},
    object::ObjectCommand,
    resource::ResourceCommand,
    server::ServerCommand,
};

use commands::init::Init;
use rooch_types::cli::{CliResult, CommandAction};

pub mod commands;

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
    #[clap(subcommand)]
    Server(ServerCommand),
    Resource(ResourceCommand),
    Object(ObjectCommand),
}

pub async fn run_cli(opt: RoochCli) -> CliResult<String> {
    match opt.cmd {
        Command::Move(move_cli) => move_cli.execute().await,
        Command::Server(server) => server.execute().await,
        Command::Resource(resource) => resource.execute_serialized().await,
        Command::Object(object) => object.execute_serialized().await,
        Command::Init(c) => c.execute_serialized().await,
        Command::Account(a) => a.execute_serialized().await,
    }
}

async fn prompt_if_no_config() -> Result<RoochConfig, anyhow::Error> {
    let config_path = rooch_config_path().map_err(CliError::from)?;

    if !config_path.exists() {
        println!(
            "Creating config file [{:?}] with default server and ed25519 key scheme.",
            config_path
        );

        crate::commands::init::init().await?;
    }

    Ok(PersistedConfig::read(&config_path)
        .map_err(|err| CliError::ConfigLoadError(format!("{:?}", config_path), err.to_string()))?)
}
