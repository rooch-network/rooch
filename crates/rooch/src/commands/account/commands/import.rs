// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use std::fmt::Debug;

use async_trait::async_trait;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{
    crypto::BuiltinScheme::Ed25519,
    error::{RoochError, RoochResult},
};

use crate::types::{CommandAction, WalletContextOptions};

/// Add a new key to rooch.keystore based on the input mnemonic phrase
#[derive(Debug, Parser)]
pub struct ImportCommand {
    mnemonic_phrase: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for ImportCommand {
    async fn execute(self) -> RoochResult<()> {
        println!("{:?}", self.mnemonic_phrase);

        let mut context = self.context_options.build().await?;

        let address = context
            .config
            .keystore
            .import_from_mnemonic(&self.mnemonic_phrase, Ed25519, None)
            .map_err(|e| RoochError::ImportAccountError(e.to_string()))?;

        println!("Key imported for address [{address}]");

        Ok(())
    }
}
