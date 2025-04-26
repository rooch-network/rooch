// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::state::MoveState;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::ParsedAddress,
    crypto::SignatureScheme,
    error::RoochResult,
    framework::auth_payload::{SignData, MESSAGE_INFO_PREFIX},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountSignView {
    pub signature: String,
    pub schnorr_signature: Option<String>,
}

impl AccountSignView {
    pub fn new(signature: String, schnorr_signature: Option<String>) -> Self {
        Self {
            signature,
            schnorr_signature,
        }
    }
}

#[async_trait]
impl CommandAction<Option<AccountSignView>> for SignCommand {
    async fn execute(self) -> RoochResult<Option<AccountSignView>> {
        let context = self.context_options.build_require_password()?;
        let password = context.get_password();
        let mapping = context.address_mapping();
        let rooch_address = self.address.into_rooch_address(&mapping)?;

        let sign_data =
            SignData::new_without_tx_hash(MESSAGE_INFO_PREFIX.to_vec(), self.message.to_bytes());
        let encoded_sign_data = sign_data.encode();

        // Secp256k1 Ecdsa or Ed25519 signature
        let sig =
            context
                .keystore
                .sign_hashed(&rooch_address, &encoded_sign_data, password.clone())?;
        let signature_bytes = sig.as_ref();
        let signature_hex = hex::encode(signature_bytes);
        // Secp256k1 Schnorr signature
        let kp = context
            .keystore
            .get_key_pair(&rooch_address, password.clone())?;
        let pubkey = kp.public();
        let schnorr_signature_hex = if pubkey.flag() == SignatureScheme::Secp256k1.flag() {
            let schnorr_sig = context.keystore.sign_schnorr(
                &rooch_address,
                &encoded_sign_data,
                password.clone(),
            )?;
            Some(hex::encode(schnorr_sig))
        } else {
            None
        };
        let account_sign_view = AccountSignView::new(signature_hex, schnorr_signature_hex);

        if self.json {
            Ok(Some(account_sign_view))
        } else {
            if account_sign_view.schnorr_signature.is_some() {
                println!(
                    "Sign message succeeded with the signatues {:?} and {:?}",
                    account_sign_view.signature,
                    account_sign_view.schnorr_signature.unwrap()
                );
            } else {
                println!(
                    "Sign message succeeded with the signatue {:?}",
                    account_sign_view.signature
                );
            }
            Ok(None)
        }
    }
}
