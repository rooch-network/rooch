// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use rooch_types::error::{RoochError, RoochResult};
use std::path::PathBuf;

use self::commands::{add::AddCommand, list::ListCommand, switch::SwitchCommand};

pub mod commands;

/// Interface for managing multiple environments
#[derive(clap::Parser)]
pub struct Env {
    #[clap(subcommand)]
    cmd: EnvCommand,
    /// Sets the file storing the state of our user accounts (an empty one will be created if missing)
    #[clap(long = "client.config")]
    config: Option<PathBuf>,
}

#[async_trait]
impl CommandAction<String> for Env {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            EnvCommand::Add(add) => add.execute().await.map(|_| "".to_owned()),
            EnvCommand::List(list) => list.execute().await.map(|_| "".to_owned()),
            EnvCommand::Switch(switch) => switch.execute().await.map(|_| "".to_owned()),
        }
        .map_err(RoochError::from)
    }
}

#[derive(Debug, clap::Subcommand)]
#[clap(name = "env")]
pub enum EnvCommand {
    Add(AddCommand),
    List(ListCommand),
    Switch(SwitchCommand),
}
