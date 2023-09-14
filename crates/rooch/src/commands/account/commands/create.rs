// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{account::AccountModule, error::RoochResult, multichain_id::RoochMultiChainID};

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let mut context = self.context_options.build().await?;

        let (new_address, phrase, multichain_id) = context
            .config
            .keystore
            .generate_and_add_new_key(RoochMultiChainID::Rooch, None, None)?;

        println!("{}", AccountAddress::from(new_address).to_hex_literal());
        println!(
            "Generated new keypair for address with multichain id {:?} [{new_address}]",
            multichain_id.multichain_id().id().to_string()
        );
        println!("Secret Recovery Phrase : [{phrase}]");

        // Obtain account address
        let address = AccountAddress::from(new_address);

        // Create account action
        let action = AccountModule::create_account_action(address);

        let result = context
            .sign_and_execute(new_address, action, multichain_id)
            .await?;
        context.assert_execute_success(result)
    }
}
