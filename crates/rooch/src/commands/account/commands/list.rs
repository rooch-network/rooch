// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{crypto::EncodeDecodeBase64, error::RoochResult};
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
            "{0: ^66} | {1: ^48} | {2: ^16} | {3: ^12}",
            "Rooch Address (Ed25519)", "Public Key (Base64)", "Auth Validator ID", "Active Address"
        );
        println!("{}", ["-"; 153].join(""));
        for (address, public_key) in context.config.keystore.get_address_public_keys() {
            let auth_validator_id = public_key.auth_validator().flag();
            let mut active = "";
            if active_address == Some(address) {
                active = "True";
            };

            println!(
                "{0: ^66} | {1: ^48} | {2: ^16} | {3: ^12}",
                address,
                public_key.encode_base64(),
                auth_validator_id.to_string(),
                active
            );
        }

        Ok(())
    }
}
