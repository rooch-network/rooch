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
use rooch_types::{
    crypto::{RoochSignature, Signature},
    error::{RoochError, RoochResult},
};

/// Verify a signature
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    /// A signature for verify
    #[clap(short = 's', long)]
    signature_hash: String,

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
        let signature_bytes = hex::decode(self.signature_hash)
            .map_err(|e| RoochError::CommandArgumentError(format!("Decode hex failed: {}", e)))?;
        let signatrue = Signature::from_bytes(&signature_bytes).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid signature argument: {}", e))
        })?;
        let pk = Secp256k1PublicKey::from_bytes(signatrue.public_key_bytes())
            .map_err(|e| RoochError::CommandArgumentError(format!("Invalid public key: {}", e)))?;
        let sig = Secp256k1Signature::from_bytes(signatrue.signature_bytes()).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid signature argument: {}", e))
        })?;
        let message_hash = hex::decode(self.message_hash)
            .map_err(|e| RoochError::CommandArgumentError(format!("Decode hex failed: {}", e)))?;
        pk.verify_with_hash::<Sha256>(&message_hash, &sig)
            .map_err(|e| RoochError::CommandArgumentError(format!("Failed verification: {}", e)))?;

        if self.json {
            Ok(Some(true))
        } else {
            println!("Verification succeeded");
            Ok(None)
        }
    }
}
