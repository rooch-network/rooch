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
pub struct CloseCommand {
    /// Channel ID to close
    #[clap(long, help = "Channel ID to close")]
    pub channel_id: ObjectID,

    /// Hex-encoded proofs for channel closure
    #[clap(long, help = "Hex-encoded proofs for channel closure")]
    pub proofs: String,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseOutput {
    pub channel_id: ObjectID,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<CloseOutput> for CloseCommand {
    async fn execute(self) -> RoochResult<CloseOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Decode proofs from hex
        let proofs_bytes = hex::decode(&self.proofs).map_err(|e| {
            rooch_types::error::RoochError::CommandArgumentError(format!(
                "Invalid hex proofs: {}",
                e
            ))
        })?;

        // Create the close channel action
        let coin_type = RGas::struct_tag();
        let action = PaymentChannelModule::close_channel_entry_action(
            coin_type,
            self.channel_id.clone(),
            proofs_bytes,
        );

        // Execute transaction using DID account signing
        let result = context
            .sign_and_execute_as_did(sender, action, max_gas_amount)
            .await?;

        Ok(CloseOutput {
            channel_id: self.channel_id,
            execution_info: result.execution_info,
        })
    }
}
