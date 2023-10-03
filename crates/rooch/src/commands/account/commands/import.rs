// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hex::ToHex;
use std::fmt::Debug;

use async_trait::async_trait;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{
    error::{RoochError, RoochResult},
    keypair_type::KeyPairType,
};

use crate::cli_types::{CommandAction, WalletContextOptions};

/// Add a new key to rooch.keystore based on the input mnemonic phrase
#[derive(Debug, Parser)]
pub struct ImportCommand {
    #[clap(short = 'm', long = "mnemonic-phrase")]
    mnemonic_phrase: String,
    /// Whether a password should be provided
    #[clap(long = "password")]
    password_required: Option<bool>,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for ImportCommand {
    async fn execute(self) -> RoochResult<()> {
        println!("{:?}", self.mnemonic_phrase);

        let mut context = self.context_options.build().await?;

        let password = if self.password_required == Some(false) {
            // Use an empty password if not required
            String::new()
        } else {
            // Prompt for a password if required
            rpassword::prompt_password("Enter a password to encrypt the keys in the rooch keystore. Press return to have an empty value: ").unwrap()
        };
        println!("Your password is {}", password);

        let (address, password_hash, nonce, ciphertext, tag) = context
            .keystore
            .import_from_mnemonic(
                &self.mnemonic_phrase,
                KeyPairType::RoochKeyPairType,
                None,
                Some(password),
            )
            .map_err(|e| RoochError::ImportAccountError(e.to_string()))?;

        context.config.password = Some(result.encryption.hashed_password);
        context.config.nonce = Some(result.encryption.nonce.encode_hex());
        context.config.ciphertext = Some(result.encryption.ciphertext.encode_hex());
        context.config.tag = Some(result.encryption.tag.encode_hex());
        context.config.save()?;

        println!(
            "Key imported for address on type {:?}: [{}]",
            KeyPairType::RoochKeyPairType.type_of(),
            result.address
        );

        Ok(())
    }
}
