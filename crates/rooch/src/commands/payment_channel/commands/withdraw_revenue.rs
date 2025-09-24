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
use rooch_types::framework::payment_revenue::PaymentRevenueModule;
use serde::{Deserialize, Serialize};

/// Withdraw revenue from payment revenue hub
#[derive(Debug, Parser)]
pub struct WithdrawRevenueCommand {
    /// DID address of the revenue hub owner (the DID document address)
    #[clap(long, help = "DID address of the revenue hub owner")]
    pub owner: ParsedAddress,

    /// Amount to withdraw
    #[clap(long, help = "Amount to withdraw from revenue")]
    pub amount: U256,

    /// Coin type to withdraw
    #[clap(
        long,
        default_value = "0x3::gas_coin::RGas",
        value_parser=ParsedStructType::parse,
        help = "Coin type to withdraw"
    )]
    pub coin_type: ParsedStructType,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawRevenueOutput {
    pub owner: RoochAddress,
    pub amount: StrView<U256>,
    pub coin_type: String,
    pub fee_amount: StrView<U256>,
    pub net_amount: StrView<U256>,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<WithdrawRevenueOutput> for WithdrawRevenueCommand {
    async fn execute(self) -> RoochResult<WithdrawRevenueOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Resolve revenue hub owner address (can be different from sender)
        let owner_address = context.resolve_rooch_address(self.owner)?;

        // Note: sender can be different from owner (e.g., DID account controlling regular account)

        // Create the withdraw revenue action
        let coin_type = self.coin_type.into_struct_tag(&context.address_mapping())?;
        let action =
            PaymentRevenueModule::withdraw_revenue_entry_action(coin_type.clone(), self.amount);

        // Execute transaction with automatic address type detection
        let result = context
            .sign_and_execute_action(sender, action, max_gas_amount)
            .await?;

        // For now, no fees are charged, so net amount equals gross amount
        let fee_amount = U256::zero();
        let net_amount = self.amount;

        Ok(WithdrawRevenueOutput {
            owner: owner_address,
            amount: self.amount.into(),
            coin_type: coin_type.to_string(),
            fee_amount: fee_amount.into(),
            net_amount: net_amount.into(),
            execution_info: result.execution_info,
        })
    }
}
