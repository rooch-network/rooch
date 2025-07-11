// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::{ArgGroup, Parser};
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::{StrView, TransactionExecutionInfoView};
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::payment_channel::PaymentChannelModule;
use rooch_types::framework::payment_channel::SignedSubRav;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[clap(group(
    ArgGroup::new("rav_input")
        .required(true)
        .args(&["rav_encoded", "individual_params"])
))]
pub struct ClaimCommand {
    /// Multibase encoded signed RAV string (alternative to individual parameters)
    #[clap(
        long,
        help = "Multibase encoded signed RAV string from create-rav command",
        group = "rav_input"
    )]
    pub rav_encoded: Option<String>,

    /// Use individual parameters instead of encoded RAV string
    #[clap(
        long,
        help = "Use individual parameters (requires all individual params)",
        group = "rav_input"
    )]
    pub individual_params: bool,

    /// Channel ID to claim from
    #[clap(
        long,
        help = "Channel ID to claim from",
        required_if_eq("individual_params", "true")
    )]
    pub channel_id: Option<ObjectID>,

    /// VM ID fragment
    #[clap(
        long,
        help = "VM ID fragment for the sub-channel",
        required_if_eq("individual_params", "true")
    )]
    pub vm_id_fragment: Option<String>,

    /// Amount to claim
    #[clap(
        long,
        help = "Amount to claim from the channel",
        required_if_eq("individual_params", "true")
    )]
    pub amount: Option<U256>,

    /// Nonce for the claim
    #[clap(
        long,
        help = "Nonce for the claim",
        required_if_eq("individual_params", "true")
    )]
    pub nonce: Option<u64>,

    /// Signature in hex format
    #[clap(
        long,
        help = "Signature in hex format",
        required_if_eq("individual_params", "true")
    )]
    pub signature: Option<String>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimOutput {
    pub channel_id: ObjectID,
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
        let (channel_id, vm_id_fragment, amount, nonce, signature_bytes) =
            if let Some(encoded) = &self.rav_encoded {
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
                    signed_rav.sub_rav.vm_id_fragment,
                    signed_rav.sub_rav.amount,
                    signed_rav.sub_rav.nonce,
                    signature_bytes,
                )
            } else if self.individual_params {
                // Use individual parameters
                let channel_id = self.channel_id.ok_or_else(|| {
                    rooch_types::error::RoochError::CommandArgumentError(
                        "Channel ID is required when using individual parameters".to_string(),
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

                (channel_id, vm_id_fragment, amount, nonce, signature_bytes)
            } else {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    "Either --rav-encoded or --individual-params must be provided".to_string(),
                ));
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
            vm_id_fragment,
            amount: amount.into(),
            nonce,
            claimer,
            execution_info: result.execution_info,
        })
    }
}
