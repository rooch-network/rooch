// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::MoveStructType;
use rooch_rpc_api::jsonrpc_types::TransactionExecutionInfoView;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::gas_coin::RGas;
use rooch_types::framework::payment_channel::PaymentChannelModule;
use serde::{Deserialize, Serialize};

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};

#[derive(Debug, Parser)]
pub struct FinalizeCancellationCommand {
    /// Channel ID to finalize cancellation for
    #[clap(long, help = "Channel ID to finalize cancellation for")]
    pub channel_id: ObjectID,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeCancellationOutput {
    pub channel_id: ObjectID,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<FinalizeCancellationOutput> for FinalizeCancellationCommand {
    async fn execute(self) -> RoochResult<FinalizeCancellationOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Create the finalize cancellation action
        let coin_type = RGas::struct_tag();
        let action = PaymentChannelModule::finalize_cancellation_entry_action(
            coin_type,
            self.channel_id.clone(),
        );

        // Execute the transaction
        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;
        let result = context.sign_and_execute(sender, tx_data).await?;
        context.assert_execute_success(result.clone())?;

        Ok(FinalizeCancellationOutput {
            channel_id: self.channel_id,
            execution_info: result.execution_info,
        })
    }
} 