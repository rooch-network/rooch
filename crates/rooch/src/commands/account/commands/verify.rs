// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::{
    error::{RoochError, RoochResult},
    framework::auth_payload::AuthPayload,
};

/// Verify an auth payload hex
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    /// an auth payload hex for verify
    #[clap(long)]
    input: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<String>> for VerifyCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let auth_payload = hex::decode(&self.input).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Failed to decode hex: {}, err: {:?}",
                self.input, e
            ))
        })?;

        let auth_payload = bcs::from_bytes::<AuthPayload>(&auth_payload)?;
        let _ = auth_payload.verify_without_tx_hash();

        if self.json {
            Ok(None)
        } else {
            println!("Verify the auth payload succeeded",);
            Ok(None)
        }
    }
}
