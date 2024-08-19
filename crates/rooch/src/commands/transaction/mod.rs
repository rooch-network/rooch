// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::transaction::commands::{
    build::BuildCommand, get_transactions_by_hash::GetTransactionsByHashCommand,
    get_transactions_by_order::GetTransactionsByOrderCommand,
};
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use rooch_types::error::RoochResult;

pub mod commands;

/// Tool for interacting with transaction
#[derive(Parser)]
pub struct Transaction {
    #[clap(subcommand)]
    cmd: TransactionCommand,
}

#[async_trait]
impl CommandAction<String> for Transaction {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            TransactionCommand::GetTransactionsByOrder(cmd) => cmd.execute_serialized().await,
            TransactionCommand::GetTransactionsByHash(cmd) => cmd.execute_serialized().await,
            TransactionCommand::Build(cmd) => cmd.execute_serialized().await,
        }
    }
}

#[derive(Subcommand)]
pub enum TransactionCommand {
    Build(BuildCommand),
    GetTransactionsByOrder(GetTransactionsByOrderCommand),
    GetTransactionsByHash(GetTransactionsByHashCommand),
}
