// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, io::Read};

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    error::{RoochError, RoochResult},
    transaction::RoochTransaction,
};

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct SubmitCommand {
    /// Transaction data hex to be used for submitting
    #[clap(long)]
    signed_tx_hex: Option<String>,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    #[clap(flatten)]
    context: WalletContextOptions,

    /// File location for the file being read
    #[clap(long)]
    file_location: Option<String>,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for SubmitCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let context = self.context.build()?;
        let submitted_tx;

        if let Some(file_location) = self.file_location {
            let mut file = File::open(file_location)?;
            let mut signed_tx = Vec::new();
            file.read_to_end(&mut signed_tx)?;
            let tx: RoochTransaction = bcs::from_bytes(&signed_tx)
                .map_err(|_| RoochError::BcsError(format!("Invalid signed tx: {:?}", signed_tx)))?;
            submitted_tx = context.execute(tx).await?;
        } else if let Some(signed_tx_hex) = self.signed_tx_hex {
            let signed_tx = hex::decode(signed_tx_hex.clone()).map_err(|_| {
                RoochError::CommandArgumentError(format!(
                    "Invalid signed transaction hex: {}",
                    signed_tx_hex
                ))
            })?;
            let tx: RoochTransaction = bcs::from_bytes(&signed_tx)
                .map_err(|_| RoochError::BcsError(format!("Invalid signed tx: {:?}", signed_tx)))?;
            submitted_tx = context.execute(tx).await?;
        } else {
            return Err(RoochError::CommandArgumentError(
                "Argument --file-location or --signed-tx-hex are not provided".to_owned(),
            ));
        }

        Ok(submitted_tx)
    }
}
