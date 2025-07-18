// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::u256::U256;
use rooch_rpc_api::jsonrpc_types::{StrView, TransactionExecutionInfoView};
use rooch_types::address::{ParsedAddress, RoochAddress};
use rooch_types::error::RoochResult;
use rooch_types::framework::payment_channel::PaymentChannelModule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
pub struct InitCommand {
    /// Payment Hub owner address.
    ///
    /// *This is **not** the payment-channel receiver.*
    /// The owner is the account that controls a payment hub. Any user can deposit
    /// funds into any owner's hub. The deposit will either create the hub
    /// (if it does not exist yet) or top-up its balance.
    #[clap(
        long,
        value_parser = ParsedAddress::parse,
        help = "Owner address of the payment hub (anyone can deposit to any owner's hub)"
    )]
    pub owner: ParsedAddress,

    #[clap(long, help = "Amount to deposit to the payment hub")]
    pub amount: U256,

    #[clap(
        long,
        default_value = "0x3::gas_coin::RGas",
        value_parser=ParsedStructType::parse,
        help = "Coin type to deposit"
    )]
    pub coin_type: ParsedStructType,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitOutput {
    pub owner: RoochAddress,
    pub amount: StrView<U256>,
    pub coin_type: String,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<InitOutput> for InitCommand {
    async fn execute(self) -> RoochResult<InitOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Resolve hub owner address
        let owner: RoochAddress = context.resolve_address(self.owner)?.into();

        // Create the deposit action
        let coin_type = self.coin_type.into_struct_tag(&context.address_mapping())?;
        let action = PaymentChannelModule::deposit_to_hub_entry_action(
            coin_type.clone(),
            owner.into(),
            self.amount,
        );

        // Execute transaction using sender account, note: the init command's sender is not the payment hub owner
        let result = context
            .sign_and_execute_action(sender, action, max_gas_amount)
            .await?;

        Ok(InitOutput {
            owner,
            amount: self.amount.into(),
            coin_type: coin_type.to_string(),
            execution_info: result.execution_info,
        })
    }
}
