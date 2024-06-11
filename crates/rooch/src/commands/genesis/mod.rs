// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use self::commands::init::InitCommand;
use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

pub mod commands;

/// Statedb Commands
#[derive(Parser)]
pub struct Genesis {
    #[clap(subcommand)]
    cmd: GenesisCommand,
}

#[async_trait]
impl CommandAction<String> for Genesis {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            GenesisCommand::Init(init) => init.execute().await.map(|_| "".to_string()),
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "genesis")]
pub enum GenesisCommand {
    Init(InitCommand),
}
