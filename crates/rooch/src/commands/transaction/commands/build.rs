// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use moveos_types::{move_types::FunctionId, transaction::MoveAction};
use rooch_framework::MOVEOS_STD_ADDRESS;
use rooch_rpc_api::jsonrpc_types::TransactionWithInfoPageView;
use rooch_rpc_client::wallet_context;
use rooch_types::{error::RoochResult, transaction::RoochTransactionData};

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct BuildCommand {
    /// Transaction's hash
    #[clap(long)]
    pub cursor: Option<u64>,

    #[clap(long)]
    pub limit: Option<u64>,

    /// descending order
    #[clap(short = 'd', long)]
    descending_order: Option<bool>,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    #[clap(flatten)]
    context: WalletContextOptions,
}

#[async_trait]
impl CommandAction<RoochTransactionData> for BuildCommand {
    async fn execute(self) -> RoochResult<RoochTransactionData> {
        let context = self.context.build()?;
        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount = self.tx_options.max_gas_amount;

        // TODO: actions for building a tx data
        let mut bundles: Vec<Vec<u8>> = vec![];
        let args = bcs::to_bytes(&bundles).unwrap();
        let action = MoveAction::new_function_call(
            FunctionId::new(
                ModuleId::new(
                    MOVEOS_STD_ADDRESS,
                    Identifier::new("module_store".to_owned()).unwrap(),
                ),
                Identifier::new("publish_modules_entry".to_owned()).unwrap(),
            ),
            vec![],
            vec![args],
        );

        let tx_data = context.build_tx_data(sender, action, max_gas_amount);
        Ok(tx_data)
    }
}
