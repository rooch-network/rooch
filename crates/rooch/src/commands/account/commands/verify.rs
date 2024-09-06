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
use moveos_types::{h256::sha2_256_of, state::MoveState};
use rooch_types::{
    crypto::{RoochSignature, Signature},
    error::{RoochError, RoochResult},
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
        let signatrue =
            Signature::from_bytes(self.signature.into_inner().as_ref()).map_err(|e| {
                RoochError::CommandArgumentError(format!("Invalid signature argument: {}", e))
            })?;
        let pk = Secp256k1PublicKey::from_bytes(signatrue.public_key_bytes())
            .map_err(|e| RoochError::CommandArgumentError(format!("Invalid public key: {}", e)))?;
        let sig = Secp256k1Signature::from_bytes(signatrue.signature_bytes()).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid signature argument: {}", e))
        })?;

        let sign_data =
            SignData::new_without_tx_hash(MESSAGE_INFO_PREFIX.to_vec(), self.message.to_bytes());
        let encoded_sign_data = sign_data.encode();
        let message_hash = sha2_256_of(&encoded_sign_data).0.to_vec();
        
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
