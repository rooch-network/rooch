// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::{
    key_derive::{retrieve_key_pair, verify_password},
    keystore::AccountKeystore,
};
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
    framework::native_validator::NativeValidatorModule,
};
use rpassword::prompt_password;

use crate::cli_types::{CommandAction, WalletContextOptions};
use std::str::FromStr;

/// Update an address with a new keypair from coin id to rooch.keystore
#[derive(Debug, Parser)]
pub struct UpdateCommand {
    /// Rooch address in string format.
    #[clap(short = 'a', long = "address")]
    address: String,
    #[clap(short = 'm', long = "mnemonic-phrase")]
    mnemonic_phrase: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpdateCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let mut context = self.context_options.build().await?;
        let existing_address = RoochAddress::from_str(&self.address)?;

        // TODO: custom mnemonic_phrase and derivation_path are required to generate a new encryption data
        let (encryption, password) = if context.client_config.is_password_empty {
            (
                context.keystore.update_address_with_encryption_data(
                    &existing_address,
                    self.mnemonic_phrase,
                    None,
                    None,
                )?,
                None,
            )
        } else {
            let password = prompt_password(
                "Enter the password saved in client config to update address with a new encryption data:",
            )
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

            (
                context.keystore.update_address_with_encryption_data(
                    &existing_address,
                    self.mnemonic_phrase,
                    None,
                    Some(password),
                )?,
                Some(password),
            )
        };

        println!(
            "{}",
            AccountAddress::from(existing_address).to_hex_literal()
        );
        println!(
            "Updated a new encryption data for an existing address {:?}",
            existing_address
        );

        let kp = retrieve_key_pair(&encryption, password.clone())?;
        let public_key = kp.public().as_ref().to_vec();
        let action = NativeValidatorModule::rotate_authentication_key_action(public_key);
        let result = context
            .sign_and_execute(existing_address, action, password.clone())
            .await?;
        context.assert_execute_success(result);
    }
}
