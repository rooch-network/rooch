// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod commands;

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use commands::start::StartCommand;
use rooch_types::error::RoochResult;

#[derive(Parser)]
pub struct Relay {
    #[clap(subcommand)]
    cmd: RelayCommand,
}

#[async_trait]
impl CommandAction<String> for Relay {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            RelayCommand::Start(start) => start.execute_serialized().await,
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "relay")]
pub enum RelayCommand {
    Start(StartCommand),
}
