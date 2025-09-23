// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::TransactionExecutionInfoView;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::payment_channel::{PaymentChannelModule, SignedSubRav};
use serde::{Deserialize, Serialize};

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};

#[derive(Debug, Parser)]
pub struct DisputeCommand {
    /// Channel ID to dispute
    #[clap(long, help = "Channel ID to dispute")]
    pub channel_id: ObjectID,

    /// Multibase encoded signed RAV from create-rav command (alternative to individual parameters)
    #[clap(long, help = "Multibase encoded signed RAV from create-rav command", conflicts_with_all = &["vm_id_fragment", "dispute_amount", "dispute_nonce", "signature"])]
    pub rav: Option<String>,

    /// VM ID fragment for the dispute
    #[clap(long, help = "VM ID fragment for the dispute", requires_all = &["dispute_amount", "dispute_nonce", "signature"])]
    pub vm_id_fragment: Option<String>,

    /// Dispute amount
    #[clap(long, help = "Dispute amount")]
    pub dispute_amount: Option<U256>,

    /// Dispute nonce
    #[clap(long, help = "Dispute nonce")]
    pub dispute_nonce: Option<u64>,

    /// Hex-encoded signature for the dispute
    #[clap(long, help = "Hex-encoded signature for the dispute")]
    pub signature: Option<String>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeOutput {
    pub channel_id: ObjectID,
    pub vm_id_fragment: String,
    pub dispute_amount: U256,
    pub dispute_nonce: u64,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<DisputeOutput> for DisputeCommand {
    async fn execute(self) -> RoochResult<DisputeOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Parse parameters either from RAV or individual arguments
        let (vm_id_fragment, dispute_amount, dispute_nonce, signature_bytes) = if let Some(
            rav_encoded,
        ) = self.rav
        {
            // Decode RAV from multibase encoded string
            let signed_rav: SignedSubRav = SignedSubRav::decode_from_multibase(&rav_encoded)
                .map_err(|e| {
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

            // Decode signature from hex
            let signature_bytes = hex::decode(&signed_rav.signature).map_err(|e| {
                rooch_types::error::RoochError::CommandArgumentError(format!(
                    "Invalid hex signature in RAV: {}",
                    e
                ))
            })?;

            (
                signed_rav.sub_rav.vm_id_fragment,
                signed_rav.sub_rav.amount,
                signed_rav.sub_rav.nonce,
                signature_bytes,
            )
        } else {
            // Use individual parameters
            let vm_id_fragment = self.vm_id_fragment.ok_or_else(|| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Either --rav or individual parameters (--vm-id-fragment, --dispute-amount, --dispute-nonce, --signature) must be provided".to_string()
                )
            })?;

            let dispute_amount = self.dispute_amount.ok_or_else(|| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Either --rav or individual parameters (--vm-id-fragment, --dispute-amount, --dispute-nonce, --signature) must be provided".to_string()
                )
            })?;

            let dispute_nonce = self.dispute_nonce.ok_or_else(|| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Either --rav or individual parameters (--vm-id-fragment, --dispute-amount, --dispute-nonce, --signature) must be provided".to_string()
                )
            })?;

            let signature = self.signature.ok_or_else(|| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Either --rav or individual parameters (--vm-id-fragment, --dispute-amount, --dispute-nonce, --signature) must be provided".to_string()
                )
            })?;

            // Decode signature from hex
            let signature_bytes = hex::decode(&signature).map_err(|e| {
                rooch_types::error::RoochError::CommandArgumentError(format!(
                    "Invalid hex signature: {}",
                    e
                ))
            })?;

            (
                vm_id_fragment,
                dispute_amount,
                dispute_nonce,
                signature_bytes,
            )
        };

        // Create the dispute action
        let action = PaymentChannelModule::dispute_cancellation_entry_action(
            self.channel_id.clone(),
            vm_id_fragment.clone(),
            dispute_amount,
            dispute_nonce,
            signature_bytes,
        );

        // Execute the transaction with automatic address type detection
        let result = context
            .sign_and_execute_action(sender, action, max_gas_amount)
            .await?;

        Ok(DisputeOutput {
            channel_id: self.channel_id.clone(),
            vm_id_fragment,
            dispute_amount,
            dispute_nonce,
            execution_info: result.execution_info,
        })
    }
}
