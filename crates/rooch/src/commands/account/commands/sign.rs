// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::state::MoveState;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::ParsedAddress, crypto::Signature, error::RoochResult, framework::auth_payload::{SignData, MESSAGE_INFO_PREFIX}
};

/// Sign a message with a parsed address
#[derive(Debug, Parser)]
pub struct SignCommand {
    // An address to be used
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "")]
    address: ParsedAddress,

    /// A message to be signed
    #[clap(short = 'm', long)]
    message: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<Signature>> for SignCommand {
    async fn execute(self) -> RoochResult<Option<Signature>> {
        let context = self.context_options.build_require_password()?;
        let password = context.get_password();
        let mapping = context.address_mapping();
        let rooch_address = self.address.into_rooch_address(&mapping)?;

        let sign_data =
            SignData::new_without_tx_hash(MESSAGE_INFO_PREFIX.to_vec(), self.message.to_bytes());
        let encoded_sign_data = sign_data.encode();

        let signature =
            context
                .keystore
                .sign_hashed(&rooch_address, &encoded_sign_data, password)?;

        if self.json {
            Ok(Some(signature))
        } else {
            println!(
                "Sign message succeeded with the signatue {:?}",
                signature
            );
            Ok(None)
        }
    }
}
