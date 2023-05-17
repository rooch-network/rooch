// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// #![allow(unused_imports)]

use anyhow::{Ok, Result};
use clap::Parser;
use rooch_client::Client;
use rooch_types::account::EncodeDecodeBase64;
use rooch_types::address::RoochAddress;
use std::fmt::Debug;

use rooch_common::config::{PersistedConfig, RoochConfig};
use rooch_key::keystore::AccountKeystore;

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct ListCommand {
    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

impl ListCommand {
    pub async fn execute(self, config: &mut PersistedConfig<RoochConfig>) -> Result<()> {
        let active_address = config.active_address;

        println!(
            " {0: ^66} | {1: ^45} | {2: ^7} | {3: ^6}",
            "Rooch Address", "Public Key (Base64)", "Scheme", "Active"
        );
        println!("{}", ["-"; 134].join(""));
        for pub_key in config.keystore.keys() {
            let mut active = "";
            let address = Into::<RoochAddress>::into(&pub_key);
            if active_address == Some(address) {
                active = "True";
            };

            println!(
                " {0: ^66} | {1: ^45} | {2: ^6} | {3: ^6}",
                address,
                pub_key.encode_base64(),
                pub_key.scheme().to_string(),
                active
            );
        }

        Ok(())
    }
}
