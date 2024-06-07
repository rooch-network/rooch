// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
};
use std::fmt::Debug;
use std::str::FromStr;

/// Nullify a keypair from a selected coin id with a Rooch address in rooch.keystore
#[derive(Debug, Parser)]
pub struct NullifyCommand {
    /// Rooch address in string format.
    #[clap(short = 'a', long = "address")]
    address: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for NullifyCommand {
    async fn execute(self) -> RoochResult<()> {
        let mut context = self.context_options.build()?;

        let existing_address = RoochAddress::from_str(self.address.as_str()).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
        })?;

        println!(
            "{}",
            AccountAddress::from(existing_address).to_hex_literal()
        );

        // Remove keypair by coin id from Rooch key store after successfully executing transaction
        context
            .keystore
            .nullify_address(&existing_address)
            .map_err(|e| RoochError::NullifyAccountError(e.to_string()))?;

        println!("Dropped an existing address {:?}", existing_address,);
        Ok(())
    }
}
