// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use fastcrypto::{
    hash::Sha256,
    secp256k1::{Secp256k1PublicKey, Secp256k1Signature},
    traits::ToFromBytes,
};
use moveos_types::state::MoveState;
use rooch_types::{
    crypto::{RoochSignature, Signature},
    error::{RoochError, RoochResult},
};

/// Verify a signature
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    /// A signature for verify
    #[clap(short = 's', long)]
    signature: String,

    /// A hashed message to be verified
    #[clap(short = 'm', long)]
    message_hash: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<bool>> for VerifyCommand {
    async fn execute(self) -> RoochResult<Option<bool>> {
        let signature_bytes = self.signature.to_bytes();
        let signatrue = Signature::from_bytes(&signature_bytes).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Invalid signature argument: {}",
                e.to_string()
            ))
        })?;
        let pk = Secp256k1PublicKey::from_bytes(signatrue.public_key_bytes()).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid public key: {}", e.to_string()))
        })?;
        let sig = Secp256k1Signature::from_bytes(signatrue.signature_bytes()).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Invalid signature argument: {}",
                e.to_string()
            ))
        })?;
        let message_hash = hex::decode(self.message_hash).map_err(|e| {
            RoochError::CommandArgumentError(format!("Decode hex failed: {}", e.to_string()))
        })?;
        pk.verify_with_hash::<Sha256>(&message_hash, &sig)
            .map_err(|e| {
                RoochError::CommandArgumentError(format!("Failed verification: {}", e.to_string()))
            })?;

        if self.json {
            Ok(Some(true))
        } else {
            println!("Verification succeeded");
            Ok(Some(true))
        }
    }
}
