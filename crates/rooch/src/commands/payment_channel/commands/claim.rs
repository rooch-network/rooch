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
pub struct ClaimCommand {
    /// Channel ID to claim from
    #[clap(long, help = "Channel ID to claim from")]
    pub channel_id: ObjectID,

    /// VM ID fragment
    #[clap(long, help = "VM ID fragment for the sub-channel")]
    pub vm_id_fragment: String,

    /// Amount to claim
    #[clap(long, help = "Amount to claim from the channel")]
    pub amount: U256,

    /// Nonce for the claim
    #[clap(long, help = "Nonce for the claim")]
    pub nonce: u64,

    /// Signature in hex format
    #[clap(long, help = "Signature in hex format")]
    pub signature: String,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimOutput {
    pub channel_id: ObjectID,
    pub vm_id_fragment: String,
    pub amount: U256,
    pub nonce: u64,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<ClaimOutput> for ClaimCommand {
    async fn execute(self) -> RoochResult<ClaimOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Decode signature from hex
        let signature_bytes = hex::decode(&self.signature).map_err(|e| {
            rooch_types::error::RoochError::CommandArgumentError(format!(
                "Invalid hex signature: {}",
                e
            ))
        })?;

        // Create the claim action
        let coin_type = RGas::struct_tag();
        let action = PaymentChannelModule::claim_from_channel_entry_action(
            coin_type,
            self.channel_id.clone(),
            self.vm_id_fragment.clone(),
            self.amount,
            self.nonce,
            signature_bytes,
        );

        // Execute transaction using DID account signing
        let result = context
            .sign_and_execute_as_did(sender, action, max_gas_amount)
            .await?;

        Ok(ClaimOutput {
            channel_id: self.channel_id,
            vm_id_fragment: self.vm_id_fragment,
            amount: self.amount,
            nonce: self.nonce,
            execution_info: result.execution_info,
        })
    }
}
