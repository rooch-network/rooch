// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{address::RoochAddress, crypto::EncodeDecodeBase64, error::RoochResult};
use std::fmt::Debug;

/// List all keys by its Rooch address, Base64 encoded public key
#[derive(Debug, Parser)]
pub struct ListCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for ListCommand {
    async fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build().await?;
        let active_address = context.config.active_address;

        println!(
            "{0: ^66} | {1: ^45} | {2: ^7} | {3: ^6}",
            "Rooch Address (Ed25519)", "Public Key (Base64)", "Scheme", "Active"
        );
        println!("{}", ["-"; 134].join(""));
        for pub_key in context.config.keystore.keys() {
            let mut active = "";
            let address = Into::<RoochAddress>::into(&pub_key);
            if active_address == Some(address) {
                active = "True";
            };

            println!(
                "{0: ^66} | {1: ^45} | {2: ^6} | {3: ^6}",
                address,
                pub_key.encode_base64(),
                pub_key.scheme().to_string(),
                active
            );
        }

        Ok(())
    }
}
