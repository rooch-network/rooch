// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{coin_type::CoinID, crypto::EncodeDecodeBase64, error::RoochResult};
use std::fmt::Debug;

/// List all keys by its Rooch address, Base64 encoded public key
#[derive(Debug, Parser)]
pub struct ListCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    #[clap(short = 'c', long = "coin-id", default_value = "rooch", arg_enum)]
    pub coin_id: CoinID,
}

#[async_trait]
impl CommandAction<()> for ListCommand {
    async fn execute(self) -> RoochResult<()> {
        match self.coin_id {
            CoinID::Bitcoin => todo!(),
            CoinID::Ether => {
                let context = self.context_options.ethereum_build().await?;
                let active_address = context.config.active_address;

                println!(
                    "{0: ^66} | {1: ^48} | {2: ^6}",
                    "Ethereum Address", "Public Key (Base64)", "Active"
                );
                println!("{}", ["-"; 127].join(""));
                for (address, public_key) in context.config.keystore.get_address_public_keys() {
                    let mut active = "";
                    if active_address == Some(address) {
                        active = "True";
                    };

                    println!(
                        "{0: ^66} | {1: ^48} | {2: ^6}",
                        address,
                        public_key.encode_base64(),
                        active
                    );
                }

                Ok(())
            }
            CoinID::Nostr => todo!(),
            CoinID::Rooch => {
                let context = self.context_options.rooch_build().await?;
                let active_address = context.config.active_address;

                println!(
                    "{0: ^66} | {1: ^48} | {2: ^6}",
                    "Rooch Address", "Public Key (Base64)", "Active"
                );
                println!("{}", ["-"; 127].join(""));
                for (address, public_key) in context.config.keystore.get_address_public_keys() {
                    let mut active = "";
                    if active_address == Some(address) {
                        active = "True";
                    };

                    println!(
                        "{0: ^66} | {1: ^48} | {2: ^6}",
                        address,
                        public_key.encode_base64(),
                        active
                    );
                }

                Ok(())
            }
        }
    }
}
