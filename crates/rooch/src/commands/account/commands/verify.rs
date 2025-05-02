// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use bitcoin::key::constants::SCHNORR_SIGNATURE_SIZE;
use clap::Parser;
use moveos_types::state::MoveState;
use rooch_types::{
    error::RoochResult,
    framework::auth_payload::{SignData, MESSAGE_INFO_PREFIX},
    rooch_signature::ParsedSignature,
};

/// Verify a signature
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    /// A signature for verify
    #[clap(short = 's', long, value_parser=ParsedSignature::parse)]
    signature: ParsedSignature,

    /// An original message to be verified
    #[clap(short = 'm', long)]
    message: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<bool>> for VerifyCommand {
    async fn execute(self) -> RoochResult<Option<bool>> {
        let signature_bytes = self.signature.into_inner();
        let verify_result = if signature_bytes.len() == SCHNORR_SIGNATURE_SIZE {
            // verify secp256k1 schnorr with hex encoded message
            let msg = hex::decode(self.message)?;
            let context = self.context_options.build()?;
            let active_address = context
                .client_config
                .active_address
                .unwrap_or_else(|| panic!("Unable to get the active address"));
            let x_only_public_key = context
                .get_key_pair(&active_address)?
                .public()
                .xonly_public_key()?;
            self.signature
                .verify_schnorr(&msg, &x_only_public_key)
                .is_ok()
        } else {
            // verify secp256k1 ecdsa with encoded sign data
            let sign_data = SignData::new_without_tx_hash(
                MESSAGE_INFO_PREFIX.to_vec(),
                self.message.to_bytes(),
            );
            let encoded_sign_data = sign_data.encode();
            self.signature.verify(&encoded_sign_data).is_ok()
        };

        if self.json {
            Ok(Some(verify_result))
        } else {
            println!("Verification result: {}", verify_result);
            Ok(None)
        }
    }
}
