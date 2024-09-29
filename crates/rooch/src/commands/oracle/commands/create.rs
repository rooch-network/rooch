// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::MoveStructType;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::oracle::{NewOracleEvent, OracleModule};
use serde::{Deserialize, Serialize};

/// Create a SimpleOracle
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(long)]
    pub name: MoveString,
    #[clap(long)]
    pub url: MoveString,
    #[clap(long)]
    pub description: MoveString,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedOracle {
    pub oracle_id: ObjectID,
    pub oracle_admin_id: ObjectID,
}

#[async_trait]
impl CommandAction<CreatedOracle> for CreateCommand {
    async fn execute(self) -> RoochResult<CreatedOracle> {
        let wallet_context = self.context_options.build_require_password()?;
        let action = OracleModule::create_oracle_action(
            self.name.to_string(),
            self.url.to_string(),
            self.description.to_string(),
        );
        let sender = wallet_context
            .resolve_address(self.tx_options.sender)?
            .into();
        let tx_data = wallet_context
            .build_tx_data(sender, action, self.tx_options.max_gas_amount)
            .await?;
        let result = wallet_context.sign_and_execute(sender, tx_data).await?;
        let result = wallet_context.assert_execute_success(result)?;
        if let Some(output) = &result.output {
            for event in &output.events {
                if event.event_type.0 == NewOracleEvent::struct_tag() {
                    let new_oracle_event: NewOracleEvent = bcs::from_bytes(&event.event_data.0)?;
                    return Ok(CreatedOracle {
                        oracle_id: new_oracle_event.oracle_id,
                        oracle_admin_id: new_oracle_event.admin_id,
                    });
                }
            }
        }
        Err(RoochError::ViewFunctionError(
            "Failed to get oracle id from output event".to_string(),
        ))
    }
}
