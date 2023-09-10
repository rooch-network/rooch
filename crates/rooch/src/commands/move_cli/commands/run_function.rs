// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{ArgWithType, CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::{move_types::FunctionId, transaction::MoveAction};
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::{
    eth::TransactionReceipt, ExecuteTransactionResponseView, TypeTagView,
};
use rooch_types::{
    address::{EthereumAddress, RoochAddress},
    coin_type::CoinID,
    error::{RoochError, RoochResult},
    transaction::{ethereum::EthereumTransaction, rooch::RoochTransaction},
};
use serde::Serialize;

#[derive(Serialize)]
pub enum ExecuteResult {
    Response(ExecuteTransactionResponseView),
    Receipt(TransactionReceipt),
}

/// Run a Move function
#[derive(Parser)]
pub struct RunFunction {
    /// Function name as `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>`
    /// Example: `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::message::set_message`
    #[clap(long)]
    pub function: FunctionId,

    /// TypeTag arguments separated by spaces.
    ///
    /// Example: `u8 u16 u32 u64 u128 u256 bool address`
    #[clap(
        long = "type-args",
        takes_value(true),
        multiple_values(true),
        multiple_occurrences(true)
    )]
    pub type_args: Vec<TypeTagView>,

    /// Arguments combined with their type separated by spaces.
    ///
    /// Supported types [u8, u16, u32, u64, u128, u256, bool, object_id, string, address, vector<inner_type>]
    ///
    /// Example: `address:0x1 bool:true u8:0 u256:1234 'vector<u32>:a,b,c,d'`
    ///     address and uint can be written in short form like `@0x1 1u8 4123u256`.
    #[clap(
        long = "args",
        takes_value(true),
        multiple_values(true),
        multiple_occurrences(true)
    )]
    pub args: Vec<ArgWithType>,

    /// RPC client options.
    #[clap(flatten)]
    context: WalletContextOptions,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    /// Command line input of coin ids
    #[clap(short = 'c', long = "coin-id", default_value = "rooch", arg_enum)]
    pub coin_id: CoinID,
}

#[async_trait]
impl CommandAction<ExecuteResult> for RunFunction {
    async fn execute(self) -> RoochResult<ExecuteResult> {
        let args: Vec<Vec<u8>> = self
            .args
            .into_iter()
            .map(|arg_with_type| arg_with_type.arg)
            .collect();

        if self.tx_options.sender_account.is_none() {
            return Err(RoochError::CommandArgumentError(
                "--sender-account required".to_owned(),
            ));
        }
        match self.coin_id {
            CoinID::Bitcoin => todo!(),
            CoinID::Ether => {
                let context = self.context.ethereum_build().await?;
                let sender: EthereumAddress = context
                    .parse_account_arg(self.tx_options.sender_account.unwrap())?
                    .into();

                let action = MoveAction::new_function_call(
                    self.function,
                    self.type_args.into_iter().map(Into::into).collect(),
                    args,
                );

                let result = match (self.tx_options.authenticator, self.tx_options.session_key) {
                    (Some(authenticator), _) => {
                        let tx_data = context.build_tx_data(sender, action).await?;
                        let tx = EthereumTransaction::new(tx_data, authenticator.into());
                        context.execute(tx).await?
                    }
                    (_, Some(session_key)) => {
                        let tx_data = context.build_tx_data(sender, action).await?;
                        let tx = context
                            .config
                            .keystore
                            .sign_transaction_via_session_key(&sender, tx_data, &session_key)
                            .map_err(|e| RoochError::SignMessageError(e.to_string()))?;
                        context.execute(tx).await?
                    }
                    (None, None) => {
                        context
                            .sign_and_execute(sender, action, self.coin_id)
                            .await?
                    }
                };

                Ok(ExecuteResult::Receipt(result))
            }
            CoinID::Rooch => {
                let context = self.context.rooch_build().await?;
                let sender: RoochAddress = context
                    .parse_account_arg(self.tx_options.sender_account.unwrap())?
                    .into();

                let action = MoveAction::new_function_call(
                    self.function,
                    self.type_args.into_iter().map(Into::into).collect(),
                    args,
                );

                let result = match (self.tx_options.authenticator, self.tx_options.session_key) {
                    (Some(authenticator), _) => {
                        let tx_data = context.build_tx_data(sender, action).await?;
                        let tx = RoochTransaction::new(tx_data, authenticator.into());
                        context.execute(tx).await?
                    }
                    (_, Some(session_key)) => {
                        let tx_data = context.build_tx_data(sender, action).await?;
                        let tx = context
                            .config
                            .keystore
                            .sign_transaction_via_session_key(&sender, tx_data, &session_key)
                            .map_err(|e| RoochError::SignMessageError(e.to_string()))?;
                        context.execute(tx).await?
                    }
                    (None, None) => {
                        context
                            .sign_and_execute(sender, action, self.coin_id)
                            .await?
                    }
                };

                Ok(ExecuteResult::Response(result))
            }
            CoinID::Nostr => todo!(),
        }
    }
}
