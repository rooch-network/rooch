// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::{eth::TransactionReceipt, ExecuteTransactionResponseView};
use rooch_types::{
    account::AccountModule, address::MultiChainAddress, coin_type::CoinID, error::RoochResult,
    framework::address_mapping::AddressMapping,
};
use serde::Serialize;

use crate::cli_types::WalletContextOptions;

#[derive(Serialize)]
pub enum ExecuteResult {
    Rooch(ExecuteTransactionResponseView),
    Ethereum(TransactionReceipt),
}

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    #[clap(short = 'c', long = "coin-id", default_value = "rooch", arg_enum)]
    pub coin_id: CoinID,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<ExecuteResult> {
        match self.coin_id {
            CoinID::Bitcoin => todo!(),
            CoinID::Ether => {
                let mut context = self.context_options.ethereum_build().await?;

                let (new_address, phrase, coin_id) = context
                    .config
                    .keystore
                    .generate_and_add_new_key(CoinID::Ether, None, None)?;

                println!("{}", new_address);
                println!(
                    "Generated new keypair for address with coin id {:?} [{new_address}]",
                    coin_id.to_string()
                );
                println!("Secret Recovery Phrase : [{phrase}]");

                let client = context.get_client().await?;

                // Obtain account address
                let multichain_address = MultiChainAddress::from(new_address);
                let address = {
                    let address_mapping = client.as_module_binding::<AddressMapping>();
                    address_mapping.resovle_or_generate(multichain_address)?
                };

                // Create account action
                let action = AccountModule::create_account_action(address);

                let result = context
                    .sign_and_execute(new_address, action, coin_id)
                    .await?;
                Ok(ExecuteResult::Ethereum(
                    context.assert_execute_success(result)?,
                ))
            }
            CoinID::Nostr => todo!(),
            CoinID::Rooch => {
                let mut context = self.context_options.rooch_build().await?;

                let (new_address, phrase, coin_id) = context
                    .config
                    .keystore
                    .generate_and_add_new_key(CoinID::Rooch, None, None)?;

                println!("{}", AccountAddress::from(new_address).to_hex_literal());
                println!(
                    "Generated new keypair for address with coin id {:?} [{new_address}]",
                    coin_id.to_string()
                );
                println!("Secret Recovery Phrase : [{phrase}]");

                // Obtain account address
                let address = AccountAddress::from(new_address);

                // Create account action
                let action = AccountModule::create_account_action(address);

                let result = context
                    .sign_and_execute(new_address, action, coin_id)
                    .await?;
                Ok(ExecuteResult::Rooch(
                    context.assert_execute_success(result)?,
                ))
            }
        }
    }
}
