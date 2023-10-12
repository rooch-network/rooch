// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rpassword::prompt_password;
use std::fmt::Debug;

use async_trait::async_trait;
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::error::{RoochError, RoochResult};

use crate::cli_types::{CommandAction, WalletContextOptions};

/// Add a new key to rooch.keystore based on the input mnemonic phrase
#[derive(Debug, Parser)]
pub struct ImportCommand {
    #[clap(short = 'm', long = "mnemonic-phrase")]
    mnemonic_phrase: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for ImportCommand {
    async fn execute(self) -> RoochResult<()> {
        println!("{:?}", self.mnemonic_phrase);

        let mut context = self.context_options.build().await?;

        let result = if context.keystore.get_if_password_is_empty() {
            context
                .keystore
                .import_from_mnemonic(&self.mnemonic_phrase, None, None)?
        } else {
            let password =
                prompt_password("Enter the password to import a key pair from mnemonic phrase:")
                    .unwrap_or_default();
            let is_verified =
                verify_password(Some(password.clone()), context.keystore.get_password_hash())?;

            if !is_verified {
                return Err(RoochError::InvalidPasswordError(
                    "Password is invalid".to_owned(),
                ));
            }

            context
                .keystore
                .import_from_mnemonic(&self.mnemonic_phrase, None, Some(password))?
        };

        println!("Key imported for address [{}]", result.address);

        Ok(())
    }
}
