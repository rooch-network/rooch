// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::ObjectID;
use rooch_types::error::RoochResult;
use serde::{Deserialize, Serialize};

use crate::cli_types::{CommandAction, WalletContextOptions};
use rooch_types::address::ParsedAddress;

/// SubRAV data structure for BCS serialization
#[derive(Serialize, Deserialize)]
struct SubRAV {
    channel_id: ObjectID,
    vm_id_fragment: String,
    amount: U256,
    nonce: u64,
}

#[derive(Debug, Parser)]
pub struct CreateRavCommand {
    /// Channel ID for the RAV
    #[clap(long, help = "Channel ID for the RAV")]
    pub channel_id: ObjectID,

    /// Verification method ID fragment (optional, if not provided, will find the first available key)
    #[clap(long, help = "Verification method ID fragment")]
    pub vm_id_fragment: Option<String>,

    /// Amount for the RAV
    #[clap(long, help = "Amount for the RAV")]
    pub amount: U256,

    /// Nonce for the RAV
    #[clap(long, help = "Nonce for the RAV")]
    pub nonce: u64,

    /// Sender DID address (the DID document address to use for signing)
    #[clap(long, value_parser=ParsedAddress::parse, default_value = "default", help = "Sender DID address")]
    pub sender: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRavOutput {
    pub channel_id: ObjectID,
    pub vm_id_fragment: String,
    pub amount: U256,
    pub nonce: u64,
    pub signature: String,
    pub signer_address: String,
}

#[async_trait]
impl CommandAction<CreateRavOutput> for CreateRavCommand {
    async fn execute(self) -> RoochResult<CreateRavOutput> {
        let context = self.context_options.build_require_password()?;

        // Resolve the sender DID address from the provided parameter
        let did_address = context.resolve_rooch_address(self.sender)?;

        // Find the appropriate verification method and keypair using the abstracted method
        let (vm_id_fragment, signer_address, keypair) = context
            .find_did_verification_method_keypair(did_address, self.vm_id_fragment.as_deref())
            .await?;

        // Create SubRAV structure for signing
        let sub_rav = SubRAV {
            channel_id: self.channel_id.clone(),
            vm_id_fragment: vm_id_fragment.clone(),
            amount: self.amount,
            nonce: self.nonce,
        };

        // Sign with the found keypair
        let signature = keypair.sign(&bcs::to_bytes(&sub_rav)?);
        let signature_hex = hex::encode(signature.as_ref());

        Ok(CreateRavOutput {
            channel_id: self.channel_id,
            vm_id_fragment,
            amount: self.amount,
            nonce: self.nonce,
            signature: signature_hex,
            signer_address: signer_address.to_string(),
        })
    }
} 