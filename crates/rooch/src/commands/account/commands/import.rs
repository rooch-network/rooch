// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::key_derive::ROOCH_SECRET_KEY_PREFIX;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::{ParsedAddress, RoochAddress},
    error::{RoochError, RoochResult},
};

/// Import an external account from an address and encoded private key into Rooch Key Store.
/// The importing format should be the same as the exported addresses and private keys.
///
/// The command must be specified with -a or --address and -k or --private-key to import an
/// external account into Rooch Key Store.
#[derive(Debug, Parser)]
pub struct ImportCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "")]
    address: ParsedAddress,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<String>> for ImportCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let mut context = self.context_options.build_require_password()?;
        let password = context.get_password();
        let result = if self.address == ParsedAddress::Named("".to_owned()) {
            context.keystore.export_mnemonic_phrase(password)?
        } else {
            let mapping = context.address_mapping();
            let rooch_address: RoochAddress =
                self.address.into_rooch_address(&mapping).map_err(|e| {
                    RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
                })?;
            println!("Address to be exported: {:?}", rooch_address);
            let kp = context.keystore.get_key_pair(&rooch_address, password)?;
            let sk_bytes = kp.private();
            context.keystore.export_private_key(sk_bytes)?
        };

        if self.json {
            Ok(Some(result))
        } else {
            if result.starts_with(ROOCH_SECRET_KEY_PREFIX) {
                println!("Export succeeded with the encoded private key [{}]", result);
            } else {
                println!("Export succeeded with the mnemonic phrase [{}]", result);
            };

            Ok(None)
        }
    }
}
