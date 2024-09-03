// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    error::{RoochError, RoochResult},
    framework::auth_payload::AuthPayload,
    transaction::{Authenticator, RoochTransactionData},
};

/// Verify a tx with a auth payload
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    /// input for tx data hex
    #[clap(long, required = true)]
    input: String,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<String>> for VerifyCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let context = self.context_options.build_require_password()?;
        let password = context.get_password();
        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let kp = context.keystore.get_key_pair(&sender, password)?;

        let tx_data_bytes = hex::decode(&self.input).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Failed to decode tx hex: {}, err:{:?}",
                self.input, e
            ))
        })?;

        let tx_data = RoochTransactionData::decode(&tx_data_bytes)?;
        let auth = Authenticator::bitcoin(&kp, &tx_data);
        let auth_payload = bcs::from_bytes::<AuthPayload>(&auth.payload).unwrap();
        let _ = auth_payload.verify(&tx_data);

        if self.json {
            Ok(None)
        } else {
            println!("Verify the auth payload succeeded",);
            Ok(None)
        }
    }
}
