// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::*;
pub mod create;
pub mod import;
pub mod list;
use self::import::ImportCommand;
use crate::commands::account::{create::CreateCommand, list::ListCommand};

use crate::config::{rooch_config_dir, Config, PersistedConfig, RoochConfig, ROOCH_CONFIG};
use async_trait::async_trait;
use rooch_types::cli::{CliError, CliResult, CommandAction};
use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct Account {
    #[clap(long = "rooch.config")]
    config: Option<PathBuf>,
    #[clap(subcommand)]
    cmd: Option<AccountCommand>,
}

#[async_trait]
impl CommandAction<()> for Account {
    async fn execute(self) -> CliResult<()> {
        let config_path = self.config.unwrap_or(
            rooch_config_dir()
                .map_err(CliError::from)?
                .join(ROOCH_CONFIG),
        );

        if !config_path.exists() {
            return Err(CliError::ConfigNotFoundError(format!(
                "{:?} not found.",
                config_path
            )));
        }

        let config: RoochConfig = PersistedConfig::read(&config_path).map_err(|err| {
            CliError::ConfigLoadError(format!("{:?}", config_path), err.to_string())
        })?;

        if let Some(cmd) = self.cmd {
            cmd.execute(&mut config.persisted(&config_path)).await?;
        } else {
            // Print help
            let mut app = Account::command();
            app.build();
            app.print_help().map_err(CliError::from)?;
        }
        Ok(())
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
