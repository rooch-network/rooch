// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::db::commands::drop::DropCommand;
use crate::commands::db::commands::repair::RepairCommand;
use crate::commands::db::commands::revert::RevertCommand;
use async_trait::async_trait;
use clap::Parser;
use commands::rollback::RollbackCommand;
use rooch_types::error::RoochResult;

pub mod commands;

/// DB Commands
#[derive(Parser)]
pub struct DB {
    #[clap(subcommand)]
    cmd: DBCommand,
}

#[async_trait]
impl CommandAction<String> for DB {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            DBCommand::Revert(revert) => revert.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::Rollback(rollback) => rollback.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::Drop(drop) => drop.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::Repair(repair) => repair.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "db")]
pub enum DBCommand {
    Revert(RevertCommand),
    Rollback(RollbackCommand),
    Drop(DropCommand),
    Repair(RepairCommand),
}
