// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::{
    account::AccountCommand,
    move_cli::{self, MoveCli},
    object::ObjectCommand,
    resource::ResourceCommand,
    server::ServerCommand,
};
use crate::config::{PersistedConfig, RoochConfig};
use clap::*;
use commands::init::Init;
use config::{rooch_config_dir, Config, ROOCH_CONFIG};
use rooch_types::cli::{CliError, CliResult};
use std::path::PathBuf;

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
    #[clap(name = "account")]
    Account {
        #[clap(long = "rooch.config")]
        config: Option<PathBuf>,
        #[clap(subcommand)]
        cmd: Option<AccountCommand>,
    },
    Init(Init),
    Move(MoveCli),
    #[clap(subcommand)]
    Server(ServerCommand),
    Resource(ResourceCommand),
    Object(ObjectCommand),
}

pub async fn run_cli(opt: RoochCli) -> CliResult<()> {
    match opt.cmd {
        Command::Move(move_cli) => move_cli::run_cli(move_cli).await,
        Command::Server(server) => server.execute().await,
        Command::Resource(resource) => resource.execute().await,
        Command::Object(object) => object.execute().await,
        Command::Init(c) => c.execute().await,
        Command::Account { config, cmd } => {
            let config_path = config.unwrap_or(
                rooch_config_dir()
                    .map_err(CliError::from)?
                    .join(ROOCH_CONFIG),
            );

            if !config_path.exists() {
                println!("Use rooch init first");
                return Ok(());
            }

            let config: RoochConfig = PersistedConfig::read(&config_path).map_err(|err| {
                CliError::ConfigLoadError(format!("{:?}", config_path), err.to_string())
            })?;

            if let Some(cmd) = cmd {
                cmd.execute(&mut config.persisted(&config_path)).await?;
            } else {
                // Print help
                let mut app = Command::command();
                app.build();
                app.find_subcommand_mut("account")
                    .unwrap()
                    .print_help()
                    .map_err(CliError::from)?;
            }
            Ok(())
        }
    }
}
