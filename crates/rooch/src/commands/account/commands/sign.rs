// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use hex::ToHex;
use moveos_types::state::MoveState;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::{
    account_sign_view::{AccountSignView, AuthPayloadView},
    BytesView,
};
use rooch_types::{
    address::ParsedAddress,
    error::{RoochError, RoochResult},
};
use std::str::FromStr;

/// Sign an msg with current account private key (sign_hashed)
///
/// This operation must be specified with -a or
/// --address to export only one address with a private key.
#[derive(Debug, Parser)]
pub struct SignCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "")]
    address: ParsedAddress,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,

    /// Msg command will sign
    #[clap(long, default_value = "")]
    msg: String,
}

#[async_trait]
impl CommandAction<Option<AccountSignView>> for SignCommand {
    async fn execute(self) -> RoochResult<Option<AccountSignView>> {
        let context = self.context_options.build_require_password()?;
        let password = context.get_password();

        let mapping = context.address_mapping();
        let address = self.address.into_rooch_address(&mapping).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
        })?;

        let signature = context.keystore.sign_hashed(
            &address,
            &self.msg.clone().to_bytes(),
            password.clone(),
        )?;

        let kp = context.keystore.get_key_pair(&address, password)?;

        let auth_payload = AuthPayloadView::new(
            BytesView::from(signature.as_ref().to_vec()),
            BytesView::from_str("Bitcoin Signed Message:\n")?,
            BytesView::from_str("Rooch Transaction:\n")?,
            BytesView::from(kp.public().as_ref().to_vec()),
            address.clone().to_bech32(),
        );

        if self.json {
            Ok(Some(AccountSignView::new(self.msg.clone(), auth_payload)))
        } else {
            println!("Msg you input : {}", &self.msg);
            println!("Signature : {}", signature.encode_hex::<String>());
            Ok(None)
        }
    }
}
