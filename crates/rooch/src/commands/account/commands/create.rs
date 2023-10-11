// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::{key_derive::verify_password, keystore::AccountKeystore};
use rooch_types::error::{RoochError, RoochResult};
use rpassword::prompt_password;

/// Create a new account off-chain.
/// If an account not exist on-chain, contract will auto create the account on-chain.
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let mut context = self.context_options.build().await?;
        let result = if context.client_config.is_password_empty {
            context
                .keystore
                .generate_and_add_new_key(None, None, None)?
        } else {
            let password = prompt_password(
                "Enter the password saved in client config to create a new key pair:",
            )
            .unwrap_or_default();
            let is_verified = verify_password(
                Some(password.clone()),
                context
                    .client_config
                    .password_hash
                    .as_ref()
                    .cloned()
                    .unwrap_or_default(),
            )?;

            if !is_verified {
                return Err(RoochError::InvalidPasswordError(
                    "Password is invalid".to_owned(),
                ));
            }

            context
                .keystore
                .generate_and_add_new_key(None, None, Some(password))?
        };

        let address = AccountAddress::from(result.address).to_hex_literal();
        println!(
            "Generated new keypair for address with key pair type [{}]",
            result.address
        );
        println!("Secret Recovery Phrase : [{}]", result.result.mnemonic);

        Ok(address)
    }
}
