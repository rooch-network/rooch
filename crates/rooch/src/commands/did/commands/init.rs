// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::did::DIDModule;
use serde::{Deserialize, Serialize};

/// Initialize the DID registry
#[derive(Debug, Parser)]
pub struct InitCommand {
    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitOutput {
    pub transaction_hash: String,
    pub gas_used: u64,
    pub status: String,
}

#[async_trait]
impl CommandAction<InitOutput> for InitCommand {
    async fn execute(self) -> RoochResult<InitOutput> {
        let mut context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Create the init DID registry action
        let action = DIDModule::init_did_registry_action();

        // Execute the transaction
        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;
        let result = context.sign_and_execute(sender, tx_data).await?;
        context.assert_execute_success(result.clone())?;

        Ok(InitOutput {
            transaction_hash: result.execution_info.tx_hash.to_string(),
            gas_used: result.execution_info.gas_used.into(),
            status: "success".to_string(),
        })
    }
} 