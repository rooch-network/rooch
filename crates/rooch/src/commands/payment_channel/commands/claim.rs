// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::{StrView, TransactionExecutionInfoView};
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::payment_channel::PaymentChannelModule;
use rooch_types::framework::payment_channel::SignedSubRav;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
pub struct ClaimCommand {
    /// Multibase encoded signed RAV string (alternative to individual parameters)
    #[clap(
        long,
        help = "Multibase encoded signed RAV string from create-rav command",
        conflicts_with_all = &["channel_id", "vm_id_fragment", "channel_epoch", "chain_id", "amount", "nonce", "signature"]
    )]
    pub rav: Option<String>,

    /// Channel ID to claim from
    #[clap(
        long,
        help = "Channel ID to claim from",
        requires_all = &["vm_id_fragment", "channel_epoch", "chain_id", "amount", "nonce", "signature"]
    )]
    pub channel_id: Option<ObjectID>,

    /// VM ID fragment
    #[clap(long, help = "VM ID fragment for the sub-channel")]
    pub vm_id_fragment: Option<String>,

    /// Channel epoch for the claim
    #[clap(long, help = "Channel epoch for the claim")]
    pub channel_epoch: Option<u64>,

    /// Chain ID for the claim
    #[clap(long, help = "Chain ID for the claim")]
    pub chain_id: Option<u64>,

    /// Amount to claim
    #[clap(long, help = "Amount to claim from the channel")]
    pub amount: Option<U256>,

    /// Nonce for the claim
    #[clap(long, help = "Nonce for the claim")]
    pub nonce: Option<u64>,

    /// Signature in hex format
    #[clap(long, help = "Signature in hex format")]
    pub signature: Option<String>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimOutput {
    pub channel_id: ObjectID,
    pub chain_id: u64,
    pub channel_epoch: u64,
    pub vm_id_fragment: String,
    pub amount: StrView<U256>,
    pub nonce: u64,
    pub claimer: RoochAddress,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<ClaimOutput> for ClaimCommand {
    async fn execute(self) -> RoochResult<ClaimOutput> {
        let context = self.context_options.build_require_password()?;
        let claimer: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Parse RAV data either from encoded string or individual parameters
        let (channel_id, chain_id, channel_epoch, vm_id_fragment, amount, nonce, signature_bytes) =
            if let Some(encoded) = &self.rav {
                // Decode from multibase encoded string
                let signed_rav: SignedSubRav = SignedSubRav::decode_from_multibase(encoded)?;

                let signature_bytes = hex::decode(&signed_rav.signature).map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!(
                        "Invalid hex signature in RAV: {}",
                        e
                    ))
                })?;

                (
                    signed_rav.sub_rav.channel_id,
                    signed_rav.sub_rav.chain_id,
                    signed_rav.sub_rav.channel_epoch,
                    signed_rav.sub_rav.vm_id_fragment,
                    signed_rav.sub_rav.amount,
                    signed_rav.sub_rav.nonce,
                    signature_bytes,
                )
            } else {
                // Use individual parameters
                let channel_id = self.channel_id.ok_or_else(|| {
                    rooch_types::error::RoochError::CommandArgumentError(
                        "Channel ID is required when using individual parameters".to_string(),
                    )
                })?;
                let chain_id = self.chain_id.ok_or_else(|| {
                    rooch_types::error::RoochError::CommandArgumentError(
                        "Chain ID is required when using individual parameters".to_string(),
                    )
                })?;
                let channel_epoch = self.channel_epoch.ok_or_else(|| {
                    rooch_types::error::RoochError::CommandArgumentError(
                        "Channel epoch is required when using individual parameters".to_string(),
                    )
                })?;
                let vm_id_fragment = self.vm_id_fragment.ok_or_else(|| {
                    rooch_types::error::RoochError::CommandArgumentError(
                        "VM ID fragment is required when using individual parameters".to_string(),
                    )
                })?;
                let amount = self.amount.ok_or_else(|| {
                    rooch_types::error::RoochError::CommandArgumentError(
                        "Amount is required when using individual parameters".to_string(),
                    )
                })?;
                let nonce = self.nonce.ok_or_else(|| {
                    rooch_types::error::RoochError::CommandArgumentError(
                        "Nonce is required when using individual parameters".to_string(),
                    )
                })?;
                let signature_str = self.signature.as_ref().ok_or_else(|| {
                    rooch_types::error::RoochError::CommandArgumentError(
                        "Signature is required when using individual parameters".to_string(),
                    )
                })?;

                let signature_bytes = hex::decode(signature_str).map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!(
                        "Invalid hex signature: {}",
                        e
                    ))
                })?;

                (
                    channel_id,
                    chain_id,
                    channel_epoch,
                    vm_id_fragment,
                    amount,
                    nonce,
                    signature_bytes,
                )
            };

        // Create the claim action
        let action = PaymentChannelModule::claim_from_channel_entry_action(
            channel_id.clone(),
            vm_id_fragment.clone(),
            amount,
            nonce,
            signature_bytes,
        );

        // Execute transaction - anyone can claim on behalf of the receiver
        let result = context
            .sign_and_execute_action(claimer, action, max_gas_amount)
            .await?;

        Ok(ClaimOutput {
            channel_id,
            chain_id,
            channel_epoch,
            vm_id_fragment,
            amount: amount.into(),
            nonce,
            claimer,
            execution_info: result.execution_info,
        })
    }
}
