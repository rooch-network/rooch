// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use hex::ToHex;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::account_sign_view::AccountSignView;
use rooch_types::error::{RoochError, RoochResult};

/// Sign an msg with current account private key (sign_hashed)
///
/// This operation must be specified with -a or
/// --address to export only one address with a private key.
#[derive(Debug, Parser)]
pub struct VerifyCommand {
    /// signature that will be verified
    #[clap(long, required = true)]
    signature: String,

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
        let addrss = self.signature.into_rooch_address(&mapping).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
        })?;

        let mut msg_body = Vec::<u8>::new();
        msg_body.copy_from_slice(&auth_payload.message_prefix);
        msg_body.copy_from_slice(&self.msg.clone().into_bytes());

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
