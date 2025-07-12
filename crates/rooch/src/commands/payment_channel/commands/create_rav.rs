// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::StrView;
use rooch_types::address::{ParsedAddress, RoochAddress};
use rooch_types::crypto::CompressedSignature;
use rooch_types::error::RoochResult;
use rooch_types::framework::payment_channel::{PaymentChannel, SignedSubRav, SubRAV};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
pub struct CreateRavCommand {
    /// Channel ID for the RAV
    #[clap(long, help = "Channel ID for the RAV")]
    pub channel_id: ObjectID,

    /// Channel epoch for the RAV (optional, if not provided, will query from chain)
    #[clap(
        long,
        help = "Channel epoch for the RAV (optional, auto-queried if not provided)"
    )]
    pub channel_epoch: Option<u64>,

    /// Chain ID for the RAV (optional, if not provided, will query from chain)
    #[clap(
        long,
        help = "Chain ID for the RAV (optional, auto-queried if not provided)"
    )]
    pub chain_id: Option<u64>,

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
    #[clap(long, value_parser=ParsedAddress::parse, help = "Sender DID address")]
    pub sender: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Serialize, Deserialize)]
pub struct SubRavView {
    pub chain_id: u64,
    pub channel_id: ObjectID,
    pub channel_epoch: u64,
    pub vm_id_fragment: String,
    pub amount: StrView<U256>,
    pub nonce: u64,
}

impl From<SubRAV> for SubRavView {
    fn from(sub_rav: SubRAV) -> Self {
        SubRavView {
            chain_id: sub_rav.chain_id,
            channel_id: sub_rav.channel_id,
            channel_epoch: sub_rav.channel_epoch,
            vm_id_fragment: sub_rav.vm_id_fragment,
            amount: sub_rav.amount.into(),
            nonce: sub_rav.nonce,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SignedSubRavView {
    pub sub_rav: SubRavView,
    pub signature: String,
}

impl From<SignedSubRav> for SignedSubRavView {
    fn from(signed_rav: SignedSubRav) -> Self {
        SignedSubRavView {
            sub_rav: signed_rav.sub_rav.into(),
            signature: signed_rav.signature,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SignedSubRavOutput {
    pub rav_hex: String,
    pub signed_rav: SignedSubRavView,
    /// Multibase encoded string containing the entire SignedSubRAV for easy copy-paste
    pub encoded: String,
    /// The local address that holds the did verification method private key for signing
    pub signer_address: RoochAddress,
}

#[async_trait]
impl CommandAction<SignedSubRavOutput> for CreateRavCommand {
    async fn execute(self) -> RoochResult<SignedSubRavOutput> {
        let context = self.context_options.build_require_password()?;

        // Resolve the sender DID address from the provided parameter
        let did_address = context.resolve_rooch_address(self.sender)?;

        // Query channel_epoch from chain if not provided
        let channel_epoch = if let Some(epoch) = self.channel_epoch {
            epoch
        } else {
            // Query the channel from chain to get current epoch
            let client = context.get_client().await?;
            let mut channel_object_views = client
                .rooch
                .get_object_states(vec![self.channel_id.clone()], None)
                .await?;

            if channel_object_views.is_empty() || channel_object_views.first().unwrap().is_none() {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    format!("Payment channel {} not found", self.channel_id),
                ));
            }

            let channel_object_view = channel_object_views.pop().unwrap().unwrap();
            let payment_channel = bcs::from_bytes::<PaymentChannel>(&channel_object_view.value.0)
                .map_err(|_| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Failed to deserialize PaymentChannel".to_string(),
                )
            })?;

            payment_channel.channel_epoch()
        };

        // Query chain_id from chain if not provided
        let chain_id = if let Some(id) = self.chain_id {
            id
        } else {
            // Query the chain_id from chain
            let client = context.get_client().await?;
            client.rooch.get_chain_id().await?
        };

        // Find the appropriate verification method and keypair using the abstracted method
        let (vm_id_fragment, signer_address, keypair) = context
            .find_did_verification_method_keypair(did_address, self.vm_id_fragment.as_deref())
            .await?;

        // Create SubRAV structure for signing
        let sub_rav = SubRAV {
            chain_id,
            channel_id: self.channel_id.clone(),
            channel_epoch,
            vm_id_fragment: vm_id_fragment.clone(),
            amount: self.amount,
            nonce: self.nonce,
        };

        let sub_rav_bytes = bcs::to_bytes(&sub_rav)?;

        // Sign with the found keypair and export **compressed** raw signature bytes (no scheme flag or public key)
        // The on-chain Move verifier expects a 64-byte ECDSA signature (r||s).
        let signature = keypair.sign(&sub_rav_bytes);

        // Convert to compressed form to strip scheme flag and embedded public key
        let compressed: CompressedSignature = signature.to_compressed()?;
        let signature_hex = hex::encode(compressed.as_ref());

        let signed_rav = SignedSubRav {
            sub_rav,
            signature: signature_hex,
        };

        let encoded = signed_rav.encode_to_multibase()?;

        let output = SignedSubRavOutput {
            rav_hex: hex::encode(sub_rav_bytes),
            signed_rav: signed_rav.into(),
            encoded,
            signer_address,
        };

        Ok(output)
    }
}
