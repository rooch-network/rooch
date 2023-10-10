// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::{key_derive::verify_password, keystore::AccountKeystore};
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rpassword::prompt_password;
use std::fmt::Debug;

use async_trait::async_trait;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
    framework::native_validator::NativeValidatorModule,
};

use crate::cli_types::{CommandAction, WalletContextOptions};
use std::str::FromStr;

/// Nullify a keypair from a selected coin id with a Rooch address in rooch.keystore
#[derive(Debug, Parser)]
pub struct NullifyCommand {
    /// Rooch address in string format.
    #[clap(short = 'a', long = "address")]
    address: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for NullifyCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let mut context = self.context_options.build().await?;

        let existing_address = RoochAddress::from_str(self.address.as_str()).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
        })?;

        println!(
            "{}",
            AccountAddress::from(existing_address).to_hex_literal()
        );

        // Create MoveAction from validator
        let action = NativeValidatorModule::remove_authentication_key_action();

        // Execute the Move call as a transaction
        let mut result = if context.client_config.is_password_empty {
            context
                .sign_and_execute(existing_address, action, None)
                .await?
        } else {
            let password =
                prompt_password("Enter the password saved in client config to delete the address:")
                    .unwrap_or_default();
            let is_verified = verify_password(
                Some(password.clone()),
                context.client_config.password_hash.unwrap_or_default(),
            )?;

            if !is_verified {
                return Err(RoochError::InvalidPasswordError(
                    "Password is invalid".to_owned(),
                ));
            }

            context
                .sign_and_execute(existing_address, action, Some(password))
                .await?
        };
        result = context.assert_execute_success(result)?;

        // Remove keypair by coin id from Rooch key store after successfully executing transaction
        context
            .keystore
            .nullify_address(&existing_address)
            .map_err(|e| RoochError::NullifyAccountError(e.to_string()))?;

        println!("Dropped an existing address {:?}", existing_address,);

        // Return transaction result
        Ok(result)
    }
}
