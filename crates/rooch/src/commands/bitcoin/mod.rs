// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use broadcast_tx::BroadcastTx;
use build_tx::BuildTx;
use clap::{Parser, Subcommand};
use rooch_types::error::RoochResult;
use sign_tx::SignTx;

pub mod broadcast_tx;
pub mod build_tx;
pub mod sign_tx;
pub mod transaction_builder;
pub mod utxo_selector;

#[derive(Debug, Parser)]
pub struct Bitcoin {
    #[clap(subcommand)]
    cmd: BitcoinCommands,
}

#[derive(Debug, Subcommand)]
pub enum BitcoinCommands {
    BuildTx(BuildTx),
    SignTx(SignTx),
    BroadcastTx(BroadcastTx),
}

#[async_trait]
impl CommandAction<String> for Bitcoin {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            BitcoinCommands::BuildTx(build_tx) => build_tx.execute().await,
            BitcoinCommands::SignTx(sign_tx) => sign_tx.execute().await,
            BitcoinCommands::BroadcastTx(broadcast_tx) => broadcast_tx.execute().await,
        }
    }
}
