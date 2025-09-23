// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::TransactionExecutionInfoView;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::payment_channel::{
    CloseProof, CloseProofs, PaymentChannelModule, SignedSubRav,
};
use serde::{Deserialize, Serialize};

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};

#[derive(Debug, Parser)]
pub struct CloseCommand {
    /// Channel ID to close
    #[clap(long, help = "Channel ID to close")]
    pub channel_id: ObjectID,

    /// Multibase encoded signed RAV strings for channel closure
    #[clap(
        long,
        help = "Multibase encoded signed RAV strings from create-rav command (can be specified multiple times)"
    )]
    pub ravs: Option<Vec<String>>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseOutput {
    pub channel_id: ObjectID,
    pub ravs_count: usize,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<CloseOutput> for CloseCommand {
    async fn execute(self) -> RoochResult<CloseOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Parse proofs data either from RAV strings or legacy hex proofs
        let (proofs_bytes, ravs_count) = if let Some(rav_strings) = &self.ravs {
            if rav_strings.is_empty() {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    "At least one RAV must be provided".to_string(),
                ));
            }

            // Decode RAVs from multibase encoded strings
            let mut close_proofs: Vec<CloseProof> = Vec::new();
            for encoded in rav_strings {
                let signed_rav: SignedSubRav = SignedSubRav::decode_from_multibase(encoded)
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

                let signature_bytes = hex::decode(&signed_rav.signature).map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!(
                        "Invalid hex signature in RAV: {}",
                        e
                    ))
                })?;

                let close_proof = CloseProof {
                    vm_id_fragment: signed_rav.sub_rav.vm_id_fragment,
                    accumulated_amount: signed_rav.sub_rav.amount,
                    nonce: signed_rav.sub_rav.nonce,
                    sender_signature: signature_bytes,
                };

                close_proofs.push(close_proof);
            }

            let close_proofs_container = CloseProofs {
                proofs: close_proofs,
            };

            let proofs_bytes = bcs::to_bytes(&close_proofs_container).map_err(|e| {
                rooch_types::error::RoochError::CommandArgumentError(format!(
                    "Failed to serialize close proofs: {}",
                    e
                ))
            })?;

            (proofs_bytes, close_proofs_container.proofs.len())
        } else {
            let close_proofs_container = CloseProofs { proofs: vec![] };

            let proofs_bytes = bcs::to_bytes(&close_proofs_container).map_err(|e| {
                rooch_types::error::RoochError::CommandArgumentError(format!(
                    "Failed to serialize close proofs: {}",
                    e
                ))
            })?;

            (proofs_bytes, 0)
        };

        // Create the close channel action
        let action =
            PaymentChannelModule::close_channel_entry_action(self.channel_id.clone(), proofs_bytes);

        let result = context
            .sign_and_execute_as_did(sender, action, max_gas_amount)
            .await?;

        Ok(CloseOutput {
            channel_id: self.channel_id,
            ravs_count,
            execution_info: result.execution_info,
        })
    }
}
