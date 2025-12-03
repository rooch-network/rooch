// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::TransactionExecutionInfoView;
use rooch_types::address::ParsedAddress;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::payment_channel::PaymentChannelModule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
pub struct OpenCommand {
    /// Channel receiver address
    #[clap(long, help = "Channel receiver address")]
    pub receiver: ParsedAddress,

    #[clap(
        long,
        help = "Coin type to use for the channel",
        value_parser=ParsedStructType::parse,
        default_value = "0x3::gas_coin::RGas"
    )]
    pub coin_type: ParsedStructType,

    #[clap(
        long,
        help = "DID verification method ID fragment for sub-channels (optional, auto-discovered from DID if not provided)"
    )]
    pub vm_id_fragment: Option<String>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOutput {
    pub receiver: RoochAddress,
    pub vm_id_fragment: String,
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

        // Parse or discover VM ID fragments
        let vm_id_fragment: String = if let Some(fragments_str) = &self.vm_id_fragment {
            // User provided fragments
            fragments_str.clone()
        } else {
            // Auto-discover from DID document - only include fragments with available keys
            // Try to find any available verification method
            match context
                .find_did_verification_method_keypair(sender, None)
                .await
            {
                Ok((fragment, _controller_addr, _keypair)) => fragment,
                Err(_) => {
                    return Err(rooch_types::error::RoochError::CommandArgumentError(
                        "No verification methods with available keys found in DID document and no VM ID fragments provided".to_string(),
                    ));
                }
            }
        };

        let coin_type = self.coin_type.into_struct_tag(&context.address_mapping())?;

        // Create the open channel action with multiple sub-channels
        let action = PaymentChannelModule::open_channel_with_sub_channel_entry_action(
            coin_type.clone(),
            receiver.into(),
            vm_id_fragment.clone(),
        );

        // Calculate the expected channel ID for output
        let channel_id = PaymentChannelModule::calc_channel_object_id(
            &coin_type,
            sender.into(),
            receiver.into(),
        );

        // Execute the transaction with automatic address type detection
        let result = context
            .sign_and_execute_action(sender, action, max_gas_amount)
            .await?;

        let fragments_source = if self.vm_id_fragment.is_some() {
            "provided".to_string()
        } else {
            "auto_discovered".to_string()
        };

        Ok(OpenOutput {
            receiver,
            vm_id_fragment: vm_id_fragment.clone(),
            fragments_source,
            channel_id,
            execution_info: result.execution_info,
        })
    }
}
