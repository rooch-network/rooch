// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::{ArgGroup, Parser};
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::{StrView, TransactionExecutionInfoView};
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::payment_channel::{
    CancelProof, CancelProofs, PaymentChannelModule, SignedSubRav,
};
use serde::{Deserialize, Serialize};

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};

#[derive(Debug, Parser)]
#[clap(group(
    ArgGroup::new("cancellation_input")
        .args(&["rav", "individual_params"])
))]
pub struct CancelCommand {
    /// Channel ID to cancel
    #[clap(long, help = "Channel ID to cancel")]
    pub channel_id: ObjectID,

    /// Multibase encoded signed RAV string for cancellation proof (optional)
    #[clap(
        long,
        help = "Multibase encoded signed RAV string from create-rav command (optional)",
        group = "cancellation_input"
    )]
    pub rav: Option<String>,

    /// Use individual parameters for cancellation proof (optional)
    #[clap(
        short = 'i',
        long,
        help = "Use individual parameters for cancellation proof (requires all individual params)",
        group = "cancellation_input"
    )]
    pub individual_params: bool,

    /// VM ID fragment for cancellation proof
    #[clap(
        long,
        help = "VM ID fragment for the sub-channel (required if using individual params)",
        required_if_eq("individual_params", "true")
    )]
    pub vm_id_fragment: Option<String>,

    /// Amount for cancellation proof
    #[clap(
        long,
        help = "Amount for cancellation proof (required if using individual params)",
        required_if_eq("individual_params", "true")
    )]
    pub amount: Option<U256>,

    /// Nonce for cancellation proof
    #[clap(
        long,
        help = "Nonce for cancellation proof (required if using individual params)",
        required_if_eq("individual_params", "true")
    )]
    pub nonce: Option<u64>,

    /// Signature in hex format for cancellation proof
    #[clap(
        long,
        help = "Signature in hex format (required if using individual params)",
        required_if_eq("individual_params", "true")
    )]
    pub signature: Option<String>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOutput {
    pub channel_id: ObjectID,
    pub proof_provided: bool,
    pub vm_id_fragment: Option<String>,
    pub amount: Option<StrView<U256>>,
    pub nonce: Option<u64>,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<CancelOutput> for CancelCommand {
    async fn execute(self) -> RoochResult<CancelOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Parse proof data if provided
        let (action, proof_provided, vm_id_fragment, amount, nonce) = if let Some(encoded) =
            &self.rav
        {
            // Decode from multibase encoded string
            let signed_rav: SignedSubRav =
                SignedSubRav::decode_from_multibase(encoded).map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!(
                        "Invalid multibase encoded RAV: {}",
                        e
                    ))
                })?;

            // Validate that the RAV is for the correct channel
            if signed_rav.sub_rav.channel_id != self.channel_id {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    format!(
                        "RAV channel ID {} does not match expected channel ID {}",
                        signed_rav.sub_rav.channel_id, self.channel_id
                    ),
                ));
            }

            // Note: For cancellation, we don't need to verify the signature
            // The signature is only validated during disputes

            // Create cancel proof structure
            let cancel_proof = CancelProof {
                vm_id_fragment: signed_rav.sub_rav.vm_id_fragment.clone(),
                accumulated_amount: signed_rav.sub_rav.amount,
                nonce: signed_rav.sub_rav.nonce,
            };
            let cancel_proofs = CancelProofs {
                proofs: vec![cancel_proof],
            };
            let serialized_proofs = bcs::to_bytes(&cancel_proofs).map_err(|e| {
                rooch_types::error::RoochError::CommandArgumentError(format!(
                    "Failed to serialize cancel proofs: {}",
                    e
                ))
            })?;

            let action = PaymentChannelModule::initiate_cancellation_with_proofs_entry_action(
                self.channel_id.clone(),
                serialized_proofs,
            );

            (
                action,
                true,
                Some(signed_rav.sub_rav.vm_id_fragment),
                Some(signed_rav.sub_rav.amount),
                Some(signed_rav.sub_rav.nonce),
            )
        } else if self.individual_params {
            // Use individual parameters
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
            let _signature_str = self.signature.as_ref().ok_or_else(|| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Signature is required when using individual parameters".to_string(),
                )
            })?;

            // Note: For cancellation, we don't need to verify the signature
            // The signature is only validated during disputes

            // Create cancel proof structure
            let cancel_proof = CancelProof {
                vm_id_fragment: vm_id_fragment.clone(),
                accumulated_amount: amount,
                nonce,
            };
            let cancel_proofs = CancelProofs {
                proofs: vec![cancel_proof],
            };
            let serialized_proofs = bcs::to_bytes(&cancel_proofs).map_err(|e| {
                rooch_types::error::RoochError::CommandArgumentError(format!(
                    "Failed to serialize cancel proofs: {}",
                    e
                ))
            })?;

            let action = PaymentChannelModule::initiate_cancellation_with_proofs_entry_action(
                self.channel_id.clone(),
                serialized_proofs,
            );

            (
                action,
                true,
                Some(vm_id_fragment),
                Some(amount),
                Some(nonce),
            )
        } else {
            // No proof provided - simple cancellation
            let action =
                PaymentChannelModule::initiate_cancellation_entry_action(self.channel_id.clone());
            (action, false, None, None, None)
        };

        // Execute transaction using DID account signing
        let result = context
            .sign_and_execute_as_did(sender, action, max_gas_amount)
            .await?;

        Ok(CancelOutput {
            channel_id: self.channel_id,
            proof_provided,
            vm_id_fragment,
            amount: amount.map(Into::into),
            nonce,
            execution_info: result.execution_info,
        })
    }
}
