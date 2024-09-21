// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, FileOrHexInput, WalletContextOptions};
use async_trait::async_trait;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    error::{RoochError, RoochResult},
    transaction::RoochTransaction,
};

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct SubmitCommand {
    /// Transaction data hex or file location to be used for submitting
    input: FileOrHexInput,

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

        let signed_tx = bcs::from_bytes::<RoochTransaction>(&self.input.data).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Invalid signed transaction hex, err: {:?}, hex: {}",
                e,
                hex::encode(&self.input.data)
            ))
        })?;

        //TODO support no json output
        let response = context.execute(signed_tx).await?;
        Ok(response)
    }
}
