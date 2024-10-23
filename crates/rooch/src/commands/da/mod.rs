// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod commands;

use crate::cli_types::CommandAction;
use crate::commands::da::commands::unpack::UnpackCommand;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

/// DB Commands
#[derive(Parser)]
pub struct DA {
    #[clap(subcommand)]
    cmd: DACommand,
}

#[async_trait]
impl CommandAction<String> for DA {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            DACommand::Unpack(unpack) => unpack.execute().map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "da")]
pub enum DACommand {
    Unpack(UnpackCommand),
}
