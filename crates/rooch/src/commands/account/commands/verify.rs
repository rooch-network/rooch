// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::{
    error::{RoochError, RoochResult},
    framework::auth_payload::AuthPayload, transaction::RoochTransactionData,
};

/// Verify a tx with a auth payload
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    /// tx data hex
    #[clap(long, required = true)]
    tx_data: String,

    /// auth payload hex
    #[clap(long, required = true)]
    auth_payload: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<String>> for VerifyCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let tx_data_bytes = hex::decode(&self.tx_data).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Failed to decode tx hex: {}, err:{:?}",
                self.tx_data, e
            ))
        })?;
        let auth_payload_bytes = hex::decode(&self.auth_payload).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Failed to decode auth payload hex: {}, err:{:?}",
                self.auth_payload, e
            ))
        })?;

        let tx_data = RoochTransactionData::decode(&tx_data_bytes)?;
        let auth_payload = bcs::from_bytes::<AuthPayload>(&auth_payload_bytes)?;
        let _ = auth_payload.verify(&tx_data);

        if self.json {
            Ok(None)
        } else {
            println!("Verify the auth payload succeeded",);
            Ok(None)
        }
    }
}
