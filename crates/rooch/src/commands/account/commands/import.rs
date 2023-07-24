// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use std::fmt::Debug;

use async_trait::async_trait;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{
    crypto::BuiltinScheme,
    error::{RoochError, RoochResult},
};

use crate::cli_types::{CommandAction, WalletContextOptions};

/// Add a new key to rooch.keystore based on the input mnemonic phrase
#[derive(Debug, Parser)]
pub struct ImportCommand {
    mnemonic_phrase: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// Command line input of crypto schemes (0 for Ed25519, 1 for MultiEd25519, 2 for Ecdsa, 3 for Schnorr)
    #[clap(short = 's', long = "scheme")]
    pub crypto_schemes: String,
}

#[async_trait]
impl CommandAction<()> for ImportCommand {
    async fn execute(self) -> RoochResult<()> {
        println!("{:?}", self.mnemonic_phrase);

        let mut context = self.context_options.build().await?;

        match BuiltinScheme::from_flag(self.crypto_schemes.clone().trim()) {
            Ok(scheme) => {
                let address = context
                    .config
                    .keystore
                    .import_from_mnemonic(&self.mnemonic_phrase, scheme, None)
                    .map_err(|e| RoochError::ImportAccountError(e.to_string()))?;

                println!("Key imported for address [{address}]");

                Ok(())
            }
            Err(error) => {
                return Err(RoochError::CommandArgumentError(format!(
                    "Invalid crypto scheme: {}",
                    error
                )))
            }
        }
    }
}
