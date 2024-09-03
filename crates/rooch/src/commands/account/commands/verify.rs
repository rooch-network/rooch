// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use hex::ToHex;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::account_sign_view::AccountSignView;
use rooch_types::error::{RoochError, RoochResult};

/// Verify a signed message
///
/// This operation must be specified with -m, or
/// --message to verify the signed message
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    /// the message to be verified
    #[clap(short = 'm', long, required = true)]
    message: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<AccountSignView>> for VerifyCommand {
    async fn execute(self) -> RoochResult<Option<AccountSignView>> {
        let context = self.context_options.build_require_password()?;
        let password = context.get_password();

        let mapping = context.address_mapping();

        self.message;
        let signature = context.keystore.sign_hashed(&addrss, &msg_body, password)?;

        if self.json {
            Ok(Some(AccountSignView::new(
                self.msg.clone(),
                signature.encode_hex(),
            )))
        } else {
            println!("Msg you input : {}", &self.msg);
            println!("Signature : {}", signature.encode_hex::<String>());
            Ok(None)
        }
    }
}
