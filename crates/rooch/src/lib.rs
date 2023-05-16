// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::{
    account::AccountCommand,
    init::Init,
    move_cli::{self, MoveCli},
    object::ObjectCommand,
    resource::ResourceCommand,
    server::ServerCommand,
};
use rooch_common::config::{
    rooch_config_dir, rooch_config_path, Config, PersistedConfig, RoochConfig, ROOCH_CONFIG,
};

use clap::*;
use rooch_types::cli::{CliError, CliResult};

pub mod commands;

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
        Command::Init(init) => init.execute().await,
        Command::Move(move_cli) => move_cli::run_cli(move_cli).await,
        Command::Server(server) => server.execute().await,
        Command::Resource(resource) => resource.execute().await,
        Command::Object(object) => object.execute().await,
        Command::Account { cmd } => {
            let config: RoochConfig = prompt_if_no_config().await?;

            if let Some(cmd) = cmd {
                cmd.execute(
                    &mut config.persisted(
                        rooch_config_dir()
                            .map_err(CliError::from)?
                            .join(ROOCH_CONFIG)
                            .as_path(),
                    ),
                )
                .await?;
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
