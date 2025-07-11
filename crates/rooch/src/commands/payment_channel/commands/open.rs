// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::MoveStructType;
use rooch_rpc_api::jsonrpc_types::TransactionExecutionInfoView;
use rooch_types::address::ParsedAddress;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::gas_coin::RGas;
use rooch_types::framework::payment_channel::PaymentChannelModule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
pub struct OpenCommand {
    /// Channel receiver address
    #[clap(long, help = "Channel receiver address")]
    pub receiver: ParsedAddress,

    /// Comma-separated list of VM ID fragments for sub-channels.
    /// If not provided, will query DID document for available verification methods.
    #[clap(
        long,
        help = "Comma-separated list of VM ID fragments for sub-channels (optional, auto-discovered from DID if not provided)"
    )]
    pub vm_id_fragments: Option<String>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOutput {
    pub receiver: RoochAddress,
    pub vm_id_fragments: Vec<String>,
    pub fragments_source: String, // "provided" or "auto_discovered"
    pub channel_id: ObjectID,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<OpenOutput> for OpenCommand {
    async fn execute(self) -> RoochResult<OpenOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Resolve receiver address
        let receiver: RoochAddress = context.resolve_address(self.receiver)?.into();

        // Validate that sender and receiver are different
        if sender == receiver {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Sender and receiver cannot be the same address".to_string(),
            ));
        }

        // Parse or discover VM ID fragments
        let vm_id_fragments: Vec<String> = if let Some(fragments_str) = &self.vm_id_fragments {
            // User provided fragments
            fragments_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // Auto-discover from DID document - only include fragments with available keys
            // Try to find any available verification method
            match context
                .find_did_verification_method_keypair(sender, None)
                .await
            {
                Ok((fragment, _controller_addr, _keypair)) => {
                    vec![fragment]
                }
                Err(_) => {
                    return Err(rooch_types::error::RoochError::CommandArgumentError(
                        "No verification methods with available keys found in DID document and no VM ID fragments provided".to_string(),
                    ));
                }
            }
        };

        if vm_id_fragments.is_empty() {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "At least one VM ID fragment is required".to_string(),
            ));
        }

        // Create the action to open channel with multiple sub-channels
        let coin_type = RGas::struct_tag();
        let action = PaymentChannelModule::open_channel_with_multiple_sub_channels_entry_action(
            coin_type.clone(),
            receiver.into(),
            vm_id_fragments.clone(),
        );

        // Execute the transaction using DID account signing
        let result = context
            .sign_and_execute_as_did(sender, action, max_gas_amount)
            .await?;

        // Calculate deterministic channel ID using the same logic as Move code
        let channel_id = PaymentChannelModule::calc_channel_object_id(
            &coin_type,
            sender.into(),
            receiver.into(),
        );

        let fragments_source = if self.vm_id_fragments.is_some() {
            "provided".to_string()
        } else {
            "auto_discovered".to_string()
        };

        Ok(OpenOutput {
            receiver,
            vm_id_fragments,
            fragments_source,
            channel_id,
            execution_info: result.execution_info,
        })
    }
}
