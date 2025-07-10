// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::MoveStructType;
use rooch_rpc_api::jsonrpc_types::TransactionExecutionInfoView;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::gas_coin::RGas;
use rooch_types::framework::payment_channel::PaymentChannelModule;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use rooch_types::address::ParsedAddress;
use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};

#[derive(Debug, Parser)]
pub struct OpenCommand {
    /// Channel receiver address
    #[clap(long, help = "Channel receiver address")]
    pub receiver: ParsedAddress,

    /// Comma-separated list of VM ID fragments for sub-channels
    #[clap(long, help = "Comma-separated list of VM ID fragments for sub-channels")]
    pub vm_id_fragments: String,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOutput {
    pub receiver: RoochAddress,
    pub vm_id_fragments: Vec<String>,
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

        // Parse VM ID fragments
        let vm_id_fragments: Vec<String> = self.vm_id_fragments
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

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

        // Execute the transaction
        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;
        let result = context.sign_and_execute(sender, tx_data).await?;
        context.assert_execute_success(result.clone())?;

        // Calculate deterministic channel ID using the same logic as Move code
        let channel_id = PaymentChannelModule::calc_channel_object_id(
            &coin_type,
            sender.into(),
            receiver.into(),
        );

        Ok(OpenOutput {
            receiver,
            vm_id_fragments,
            channel_id,
            execution_info: result.execution_info,
        })
    }
} 