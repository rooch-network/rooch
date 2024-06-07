// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::types::ParsedStructType;
use move_core_types::u256::U256;
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::address::ParsedAddress;
use rooch_types::address::RoochAddress;
use rooch_types::framework::transfer::TransferModule;
use rooch_types::{
    error::{RoochError, RoochResult},
    transaction::rooch::RoochTransaction,
};
use rpassword::prompt_password;

/// Transfer coins
#[derive(Debug, Parser)]
pub struct TransferCommand {
    /// The existing account's address to receive coins
    #[clap(short = 't', long, value_parser=ParsedAddress::parse)]
    to: ParsedAddress,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME><TypeParam>`
    /// Example: `0x3::gas_coin::GasCoin`, `0x123::Coin::Box<0x123::coin_box::FCoin>`
    #[clap(short = 'c', long, value_parser=ParsedStructType::parse)]
    coin_type: ParsedStructType,

    /// The amount of coin to transfer
    #[clap(short = 'a', long)]
    amount: U256,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    #[clap(flatten)]
    context: WalletContextOptions,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for TransferCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let context = self.context.build()?;
        let mapping = context.address_mapping();
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;
        let address_addr = self.to.into_account_address(&mapping)?;
        let coin_type = self.coin_type.into_struct_tag(&mapping)?;
        let action =
            TransferModule::create_transfer_coin_action(coin_type, address_addr, self.amount);

        match (self.tx_options.authenticator, self.tx_options.session_key) {
            (Some(authenticator), _) => {
                let tx_data = context
                    .build_tx_data(sender, action, max_gas_amount)
                    .await?;
                //TODO the authenticator usually is associated with the RoochTransactinData
                //So we need to find a way to let user generate the authenticator based on the tx_data.
                let tx = RoochTransaction::new(tx_data, authenticator.into());
                context.execute(tx).await
            }
            (_, Some(session_key)) => {
                let tx_data = context
                    .build_tx_data(sender, action, max_gas_amount)
                    .await?;
                let tx = if context.keystore.get_if_password_is_empty() {
                    context
                        .keystore
                        .sign_transaction_via_session_key(&sender, tx_data, &session_key, None)
                        .map_err(|e| RoochError::SignMessageError(e.to_string()))?
                } else {
                    let password =
                        prompt_password("Enter the password to run functions:").unwrap_or_default();
                    let is_verified = verify_password(
                        Some(password.clone()),
                        context.keystore.get_password_hash(),
                    )?;

                    if !is_verified {
                        return Err(RoochError::InvalidPasswordError(
                            "Password is invalid".to_owned(),
                        ));
                    }

                    context
                        .keystore
                        .sign_transaction_via_session_key(
                            &sender,
                            tx_data,
                            &session_key,
                            Some(password),
                        )
                        .map_err(|e| RoochError::SignMessageError(e.to_string()))?
                };
                context.execute(tx).await
            }
            (None, None) => {
                if context.keystore.get_if_password_is_empty() {
                    context
                        .sign_and_execute(sender, action, None, max_gas_amount)
                        .await
                } else {
                    let password =
                        prompt_password("Enter the password to run functions:").unwrap_or_default();
                    let is_verified = verify_password(
                        Some(password.clone()),
                        context.keystore.get_password_hash(),
                    )?;

                    if !is_verified {
                        return Err(RoochError::InvalidPasswordError(
                            "Password is invalid".to_owned(),
                        ));
                    }

                    context
                        .sign_and_execute(sender, action, Some(password), max_gas_amount)
                        .await
                }
            }
        }
    }
}
