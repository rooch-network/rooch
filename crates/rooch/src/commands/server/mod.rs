// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use commands::start::StartCommand;
use rooch_types::error::RoochResult;

pub mod commands;

#[derive(Parser)]
pub struct Server {
    #[clap(subcommand)]
    cmd: ServerCommand,
}

#[async_trait]
impl CommandAction<String> for Server {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            ServerCommand::Start(start) => start.execute_serialized().await,
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "servre")]
pub enum ServerCommand {
    Start(StartCommand),
}
