// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::state::MoveState;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::ParsedAddress,
    error::RoochResult,
    framework::auth_payload::{SignData, MESSAGE_INFO_PREFIX},
    to_bech32::PREFIX_BECH32_PUBLIC_KEY,
};

/// Sign a message with a parsed address
#[derive(Debug, Parser)]
pub struct SignCommand {
    // An address to be parsed
    #[clap(short = 'a', long = "address", default_value = "default")]
    address: String,

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
impl CommandAction<Option<String>> for SignCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let context = self.context_options.build_require_password()?;
        let password = context.get_password();
        let mapping = context.address_mapping();
        let parsed_address = ParsedAddress::from_str(&self.address)?;
        let rooch_address = parsed_address.into_rooch_address(&mapping)?;

        let signature_hex = if self.address.starts_with(PREFIX_BECH32_PUBLIC_KEY) {
            // Secp256k1 Schnorr signature
            let msg = hex::decode(self.message)?;
            let signature_bytes =
                context
                    .keystore
                    .sign_schnorr(&rooch_address, &msg, password.clone())?;
            hex::encode(signature_bytes)
        } else {
            // Secp256k1 Ecdsa or Ed25519 signature
            let sign_data = SignData::new_without_tx_hash(
                MESSAGE_INFO_PREFIX.to_vec(),
                self.message.to_bytes(),
            );
            let encoded_sign_data = sign_data.encode();
            let sig = context.keystore.sign_hashed(
                &rooch_address,
                &encoded_sign_data,
                password.clone(),
            )?;
            let signature_bytes = sig.as_ref();
            hex::encode(signature_bytes)
        };

        if self.json {
            Ok(Some(signature_hex))
        } else {
            println!(
                "Sign message succeeded with the signatue {:?}",
                signature_hex
            );
            Ok(None)
        }
    }
}
