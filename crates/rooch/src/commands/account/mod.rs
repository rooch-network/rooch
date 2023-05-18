// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::*;
pub mod create;
pub mod import;
pub mod list;
use self::import::ImportCommand;
use crate::commands::account::{create::CreateCommand, list::ListCommand};

use async_trait::async_trait;
use rooch_common::config::{
    rooch_config_dir, rooch_config_path, Config, PersistedConfig, RoochConfig, ROOCH_CONFIG,
};
use rooch_types::cli::{CliError, CliResult, CommandAction};

#[derive(clap::Parser)]
pub struct Account {
    #[clap(subcommand)]
    cmd: AccountCommand,
}

#[async_trait]
impl CommandAction<()> for Account {
    async fn execute(self) -> CliResult<()> {
        let config: RoochConfig = prompt_if_no_config().await?;

        self.cmd
            .execute(
                &mut config.persisted(
                    rooch_config_dir()
                        .map_err(CliError::from)?
                        .join(ROOCH_CONFIG)
                        .as_path(),
                ),
            )
            .await
    }
}

#[derive(Debug, clap::Subcommand)]
#[clap(name = "account")]
pub enum AccountCommand {
    Create(CreateCommand),
    // CreateResourceAccount(create_resource_account::CreateResourceAccount),
    List(ListCommand),
    Import(ImportCommand),
    // RotateKey(key_rotation::RotateKey),
}

impl AccountCommand {
    pub async fn execute(self, config: &mut PersistedConfig<RoochConfig>) -> CliResult<()> {
        match self {
            AccountCommand::Create(c) => c.execute(config).await,
            // AccountCommand::CreateResourceAccount(c) => c.execute_serialized().await,
            AccountCommand::List(c) => c.execute(config).await,
            AccountCommand::Import(c) => c.execute(config).await, // AccountCommand::RotateKey(c) => c.execute_serialized().await,
        }
        .map_err(CliError::from)
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
