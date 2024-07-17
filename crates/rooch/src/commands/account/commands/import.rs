// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use fastcrypto::{secp256k1::Secp256k1KeyPair, traits::ToFromBytes};
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::{ParsedAddress, RoochAddress},
    crypto::RoochKeyPair,
    error::{RoochError, RoochResult},
    rooch_key::ParsedSecretKey,
};

/// Import an external account from an address and encoded private key into Rooch Key Store.
/// The importing format should be the same as the exported addresses and private keys.
///
/// The command must be specified with -a or --address and -k or --secretkey to import an
/// external account into Rooch Key Store.
#[derive(Debug, Parser)]
pub struct ImportCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, required = true)]
    address: ParsedAddress,
    #[clap(short = 'k', long = "secretkey", value_parser=ParsedSecretKey::parse, required = true)]
    secretkey: ParsedSecretKey,
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
        let mapping = context.address_mapping();
        let rooch_address: RoochAddress =
            self.address.into_rooch_address(&mapping).map_err(|e| {
                RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
            })?;
        let kp = RoochKeyPair::Secp256k1(
            Secp256k1KeyPair::from_bytes(&self.secretkey.into_inner().secret_bytes()).map_err(
                |e| RoochError::CommandArgumentError(format!("Invalid Rooch secret key: {}", e)),
            )?,
        );
        context
            .keystore
            .import_external_account(rooch_address, kp, password)?;

        if self.json {
            Ok(Some(rooch_address.to_string()))
        } else {
            println!(
                "Import succeeded with address [{}] and the secret key",
                rooch_address
            );

            Ok(None)
        }
    }
}
