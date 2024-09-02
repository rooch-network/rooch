// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::is_file_path;
use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    error::{RoochError, RoochResult},
    transaction::RoochTransaction,
};
use std::{fs::File, io::Read};

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct SubmitCommand {
    /// Transaction data hex or file location to be used for submitting
    input: String,

    #[clap(flatten)]
    context: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for SubmitCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let context = self.context.build()?;

        let tx_hex = if is_file_path(&self.input) {
            let mut file = File::open(&self.input).map_err(|e| {
                RoochError::CommandArgumentError(format!(
                    "Failed to open file: {}, err:{:?}",
                    self.input, e
                ))
            })?;
            let mut hex_str = String::new();
            file.read_to_string(&mut hex_str).map_err(|e| {
                RoochError::CommandArgumentError(format!(
                    "Failed to read file: {}, err:{:?}",
                    self.input, e
                ))
            })?;
            hex_str
        } else {
            self.input
        };
        let tx_bytes = hex::decode(tx_hex.strip_prefix("0x").unwrap_or(&tx_hex)).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Invalid signed transaction hex, err: {:?}, hex: {}",
                e, tx_hex
            ))
        })?;
        let signed_tx = bcs::from_bytes::<RoochTransaction>(&tx_bytes).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Invalid signed transaction hex, err: {:?}, hex: {}",
                e, tx_hex
            ))
        })?;

        //TODO support no json output
        let response = context.execute(signed_tx).await?;
        Ok(response)
    }
}
