// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use commands::create::CreateCommand;
use rooch_types::error::RoochResult;

pub mod commands;

/// Session key Commands
#[derive(Parser)]
pub struct SessionKey {
    #[clap(subcommand)]
    cmd: SessionKeyCommand,
}

#[async_trait]
impl CommandAction<String> for SessionKey {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            SessionKeyCommand::Create(create) => create.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "session_key")]
pub enum SessionKeyCommand {
    Create(CreateCommand),
}
