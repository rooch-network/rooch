// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use std::fmt::Debug;

use async_trait::async_trait;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{
    address::RoochAddress,
    crypto::BuiltinScheme,
    error::{RoochError, RoochResult},
};

use crate::cli_types::{CommandAction, WalletContextOptions};
use std::str::FromStr;

/// Update an address with a new keypair from scheme to rooch.keystore
#[derive(Debug, Parser)]
pub struct NullifyCommand {
    /// Rooch address in string format.
    #[clap(short = 'a', long = "address")]
    address: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// Command line input of crypto schemes (ed25519, multied25519, ecdsa, or schnorr)
    #[clap(short = 's', long = "scheme", arg_enum)]
    pub crypto_schemes: BuiltinScheme,
}

#[async_trait]
impl CommandAction<()> for NullifyCommand {
    async fn execute(self) -> RoochResult<()> {
        let mut context = self.context_options.build().await?;

        match BuiltinScheme::from_flag_byte(&self.crypto_schemes.flag()) {
            Ok(scheme) => {
                let address = RoochAddress::from_str(self.address.as_str()).map_err(|e| {
                    RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
                })?;

                context
                    .config
                    .keystore
                    .nullify_address_with_key_pair_from_scheme(&address, scheme)
                    .map_err(|e| RoochError::NullifyAccountError(e.to_string()))?;

                println!("{}", AccountAddress::from(address).to_hex_literal());
                println!(
                    "Nullified a keypair from an existing address on scheme {:?} [{address}]",
                    scheme.to_owned()
                );

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
