// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::account::commands::balance::BalanceCommand;
use async_trait::async_trait;
use commands::{
    binding::BindingCommand, create::CreateCommand, list::ListCommand, nullify::NullifyCommand,
    switch::SwitchCommand,
};
use rooch_types::error::{RoochError, RoochResult};
use std::path::PathBuf;

pub mod commands;

/// Tool for interacting with accounts
#[derive(clap::Parser)]
pub struct Account {
    #[clap(subcommand)]
    cmd: AccountCommand,
    /// Sets the file storing the state of our user accounts (an empty one will be created if missing)
    #[clap(long = "client.config")]
    config: Option<PathBuf>,
}

#[async_trait]
impl CommandAction<String> for Account {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            AccountCommand::Create(create) => create.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            AccountCommand::List(list) => list.execute().await.map(|_| "".to_owned()),
            AccountCommand::Switch(switch) => switch.execute().await.map(|_| "".to_owned()),
            AccountCommand::Nullify(nullify) => nullify.execute().await.map(|_| "".to_owned()),
            AccountCommand::Balance(balance) => balance.execute().await.map(|_| "".to_owned()),
            AccountCommand::Binding(binding) => binding.execute().await.map(|_| "".to_owned()),
        }
        .map_err(RoochError::from)
    }
}

#[derive(Debug, clap::Subcommand)]
#[clap(name = "account")]
pub enum AccountCommand {
    Create(CreateCommand),
    List(ListCommand),
    Switch(SwitchCommand),
    Nullify(NullifyCommand),
    Balance(BalanceCommand),
    Binding(BindingCommand),
}
