// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::state::MoveState;
use rooch_types::{
    crypto::RoochSignature,
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
        let sign_data =
            SignData::new_without_tx_hash(MESSAGE_INFO_PREFIX.to_vec(), self.message.to_bytes());
        let encoded_sign_data = sign_data.encode();
        let verify_result = self
            .signature
            .into_inner()
            .verify(&encoded_sign_data)
            .is_ok();

        if self.json {
            Ok(Some(verify_result))
        } else {
            println!("Verification result: {}", verify_result);
            Ok(None)
        }
    }
}
