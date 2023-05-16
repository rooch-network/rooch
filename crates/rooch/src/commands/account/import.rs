// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// #![allow(unused_imports)]

use anyhow::{Ok, Result};
use clap::Parser;
use rooch_client::Client;
use std::fmt::Debug;

use rooch_common::config::{PersistedConfig, RoochConfig};
use rooch_key::keystore::AccountKeystore;
use rooch_types::account::SignatureScheme::ED25519;

/// Add a new key to sui.keystore based on the input mnemonic phrase
#[derive(Debug, Parser)]
pub struct ImportCommand {
    /// RPC client options.
    #[clap(flatten)]
    client: Client,

    mnemonic_phrase: String,
}

/// Add a new key to sui.keystore based on the input mnemonic phrase,
impl ImportCommand {
    pub async fn execute(self, config: &mut PersistedConfig<RoochConfig>) -> Result<()> {
        println!("{:?}", self.mnemonic_phrase);

        let address = config
            .keystore
            .import_from_mnemonic(&self.mnemonic_phrase, ED25519, None)?;

        println!("Key imported for address [{address}]");

        Ok(())
    }
}
