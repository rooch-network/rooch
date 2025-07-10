// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use move_core_types::u256::U256;
use moveos_types::state::MoveStructType;
use rooch_rpc_api::jsonrpc_types::TransactionExecutionInfoView;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::gas_coin::RGas;
use rooch_types::framework::payment_channel::PaymentChannelModule;
use serde::{Deserialize, Serialize};

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};

#[derive(Debug, Parser)]
pub struct InitCommand {
    /// Receiver address for the payment hub deposit
    #[clap(long, help = "Receiver address (defaults to sender address)")]
    pub receiver: Option<AccountAddress>,

    /// Amount to deposit
    #[clap(long, help = "Amount to deposit to the payment hub")]
    pub amount: U256,

    /// Coin type (defaults to RGas)
    #[clap(long, default_value = "RGas", help = "Coin type to deposit")]
    pub coin_type: String,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitOutput {
    pub receiver: AccountAddress,
    pub amount: U256,
    pub coin_type: String,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<InitOutput> for InitCommand {
    async fn execute(self) -> RoochResult<InitOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Calculate receiver address (sender unless specified)
        let receiver = self.receiver.unwrap_or(sender.into());

        // For now, we only support RGas
        if self.coin_type != "RGas" {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Currently only RGas is supported".to_string(),
            ));
        }

        // Create the deposit action
        let coin_type = RGas::struct_tag();
        let action =
            PaymentChannelModule::deposit_to_hub_entry_action(coin_type, receiver, self.amount);

        // Execute transaction using DID account signing
        let result = context
            .sign_and_execute_as_did(sender, action, max_gas_amount)
            .await?;

        Ok(InitOutput {
            receiver,
            amount: self.amount,
            coin_type: self.coin_type,
            execution_info: result.execution_info,
        })
    }
}
