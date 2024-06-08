// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::RoochAddressView;
use rooch_types::error::RoochResult;

/// Create a new account off-chain.
/// If an account not exist on-chain, contract will auto create the account on-chain.
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<RoochAddressView>> for CreateCommand {
    async fn execute(self) -> RoochResult<Option<RoochAddressView>> {
        let mut context = self.context_options.build_require_password()?;
        let password = context.get_password();
        let result = context.keystore.generate_and_add_new_key(password)?;

        if self.json {
            Ok(Some(result.address.into()))
        } else {
            println!(
                "Generated new keypair for address with key pair type [{}]",
                result.address
            );
            println!(
                "Secret Recovery Phrase : [{}]",
                result.key_pair_data.mnemonic_phrase
            );

            Ok(None)
        }
    }
}
