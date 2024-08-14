// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use commands::get_field_states::GetFieldStatesCommand;
use commands::list_field_states::ListFieldStatesCommand;
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
            DynamicFieldCommand::GetFieldStates(cmd) => cmd.execute_serialized().await,
            DynamicFieldCommand::ListFieldStates(cmd) => cmd.execute_serialized().await,
        }
    }
}

#[derive(clap::Subcommand)]
pub enum DynamicFieldCommand {
    GetFieldStates(GetFieldStatesCommand),
    ListFieldStates(ListFieldStatesCommand),
}
