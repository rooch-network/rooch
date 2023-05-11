// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod create;
pub mod import;
pub mod list;
use crate::commands::account::{create::CreateCommand, list::ListCommand};
use crate::config::{PersistedConfig, RoochConfig};
use anyhow::Result;
use rooch_types::cli::{CliError, CliResult};

use self::import::ImportCommand;

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
