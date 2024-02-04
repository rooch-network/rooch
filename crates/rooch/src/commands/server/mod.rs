// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use commands::start::StartCommand;
use rooch_types::error::RoochResult;

use self::commands::clean::CleanCommand;

pub mod commands;

/// Start Rooch network
#[derive(Parser)]
pub struct Server {
    #[clap(subcommand)]
    cmd: ServerCommand,
}

#[async_trait]
impl CommandAction<String> for Server {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            ServerCommand::Start(start) => {
                println!("{:?}", start);
                start.execute_serialized().await
            }
            ServerCommand::Clean(clean) => clean.execute().map(|_| "".to_owned()),
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "server")]
pub enum ServerCommand {
    Start(StartCommand),
    Clean(CleanCommand),
}
