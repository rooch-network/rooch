// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use hex::ToHex;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::account_sign_view::AccountSignView;
use rooch_types::{
    address::ParsedAddress,
    error::{RoochError, RoochResult},
};

/// Export an existing private key for one address or mnemonic for all addresses off-chain.
///
/// Default to export all addresses with a mnemonic phrase but can be specified with -a or
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
        let addrss = self.address.into_rooch_address(&mapping).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
        })?;

        let signature =
            context
                .keystore
                .sign_hashed(&addrss, &self.msg.clone().into_bytes(), password)?;

        if self.json {
            Ok(Some(AccountSignView::new(
                self.msg.clone(),
                signature.encode_hex(),
            )))
        } else {
            Ok(None)
        }
    }
}
