// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::RoochAddressView;
use rooch_types::address::ParsedAddress;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
};
use std::fmt::Debug;

/// Nullify a keypair from a selected coin id with a Rooch address in rooch.keystore
#[derive(Debug, Parser)]
pub struct NullifyCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse)]
    address: ParsedAddress,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<RoochAddressView>> for NullifyCommand {
    async fn execute(self) -> RoochResult<Option<RoochAddressView>> {
        let mut context = self.context_options.build()?;
        let mapping = context.address_mapping();
        let existing_address: RoochAddress =
            self.address.into_rooch_address(&mapping).map_err(|e| {
                RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
            })?;

        // Remove keypair by coin id from Rooch key store after successfully executing transaction
        context
            .keystore
            .nullify_address(&existing_address)
            .map_err(|e| RoochError::NullifyAccountError(e.to_string()))?;

        if self.json {
            Ok(Some(existing_address.into()))
        } else {
            println!(
                "{}",
                AccountAddress::from(existing_address).to_hex_literal()
            );

            println!("Dropped an existing address {:?}", existing_address,);
            Ok(None)
        }
    }
}
