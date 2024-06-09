// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::RoochAddressView;
use rooch_types::address::ParsedAddress;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
};
use std::fmt::Debug;

/// Switch the active Rooch account
#[derive(Debug, Parser)]
pub struct SwitchCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// The address of the Rooch account to be set as active
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse)]
    address: ParsedAddress,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<RoochAddressView>> for SwitchCommand {
    async fn execute(self) -> RoochResult<Option<RoochAddressView>> {
        let mut context = self.context_options.build()?;
        let mapping = context.address_mapping();
        let rooch_address: RoochAddress =
            self.address.into_rooch_address(&mapping).map_err(|e| {
                RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
            })?;

        if !context.keystore.addresses().contains(&rooch_address) {
            return Err(RoochError::SwitchAccountError(format!(
                "Address `{}` does not in the Rooch keystore",
                rooch_address
            )));
        }

        context.client_config.active_address = Some(rooch_address);
        context.client_config.save()?;

        if self.json {
            Ok(Some(rooch_address.into()))
        } else {
            println!(
                "The active account was successfully switched to `{}`",
                rooch_address
            );

            Ok(None)
        }
    }
}
