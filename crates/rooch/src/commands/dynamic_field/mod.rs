// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use commands::get::GetFieldStatesCommand;
use commands::list::ListFieldStatesCommand;
use rooch_types::error::RoochResult;

pub mod commands;

/// Dynamic Field Commands
#[derive(Parser)]
pub struct DynamicField {
    #[clap(subcommand)]
    cmd: DynamicFieldCommand,
}

#[async_trait]
impl CommandAction<String> for DynamicField {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            DynamicFieldCommand::Get(get) => get.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DynamicFieldCommand::List(list) => list.execute_serialized().await,
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "dynamic_field")]
pub enum DynamicFieldCommand {
    Get(GetFieldStatesCommand),
    List(ListFieldStatesCommand),
}
