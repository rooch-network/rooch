// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use move_core_types::u256::U256;
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
pub struct DisputeCommand {
    /// Channel ID to dispute
    #[clap(long, help = "Channel ID to dispute")]
    pub channel_id: ObjectID,

    /// VM ID fragment for the dispute
    #[clap(long, help = "VM ID fragment for the dispute")]
    pub vm_id_fragment: String,

    /// Dispute amount
    #[clap(long, help = "Dispute amount")]
    pub dispute_amount: U256,

    /// Dispute nonce
    #[clap(long, help = "Dispute nonce")]
    pub dispute_nonce: u64,

    /// Hex-encoded signature for the dispute
    #[clap(long, help = "Hex-encoded signature for the dispute")]
    pub signature: String,

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

        // Decode signature from hex
        let signature_bytes = hex::decode(&self.signature)
            .map_err(|e| rooch_types::error::RoochError::CommandArgumentError(
                format!("Invalid hex signature: {}", e)
            ))?;

        // Create the dispute action
        let coin_type = RGas::struct_tag();
        let action = PaymentChannelModule::dispute_cancellation_entry_action(
            coin_type,
            self.channel_id.clone(),
            self.vm_id_fragment.clone(),
            self.dispute_amount,
            self.dispute_nonce,
            signature_bytes,
        );

        // Execute transaction using DID account signing
        let result = context.sign_and_execute_as_did(sender, action, max_gas_amount).await?;

        Ok(DisputeOutput {
            channel_id: self.channel_id,
            vm_id_fragment: self.vm_id_fragment,
            dispute_amount: self.dispute_amount,
            dispute_nonce: self.dispute_nonce,
            execution_info: result.execution_info,
        })
    }
} 