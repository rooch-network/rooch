// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{account::AccountModule, coin_type::CoinID, error::RoochResult};

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    #[clap(short = 'c', long = "coin_id", arg_enum)]
    pub coin_id: CoinID,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        match self.coin_id {
            CoinID::Bitcoin => todo!(),
            CoinID::Ether => todo!(),
            // CoinID::Ether => {
            //     let mut context = self.context_options.ethereum_build().await?;

            //     let (new_address, phrase, coin_id) =
            //         context
            //             .config
            //             .keystore
            //             .generate_and_add_new_key(CoinID::Ether, None, None)?;

            //     println!("{}", new_address);
            //     println!(
            //         "Generated new keypair for address with scheme {:?} [{new_address}]",
            //         coin_id.to_string()
            //     );
            //     println!("Secret Recovery Phrase : [{phrase}]");

            //     let mca = MultiChainAddress::from(new_address);

            //     let address_mapping = self.moveos.as_module_binding::<AddressMapping>();

            //     let resolved_address = address_mapping::resovle_or_generate(&self, mca)?;

            //     // Obtain account address
            //     let address = AddressMapping::resovle_or_generate(self, resolved_address)?;

            //     // Create account action
            //     let action = AccountModule::create_account_action(address);

            //     let result = context
            //         .sign_and_execute(new_address, action, scheme)
            //         .await?;
            //     context.assert_execute_success(result)
            // },
            CoinID::Nostr => todo!(),
            CoinID::Rooch => {
                let mut context = self.context_options.rooch_build().await?;

                let (new_address, phrase, scheme) = context
                    .config
                    .keystore
                    .generate_and_add_new_key(CoinID::Rooch, None, None)?;

                println!("{}", AccountAddress::from(new_address).to_hex_literal());
                println!(
                    "Generated new keypair for address with scheme {:?} [{new_address}]",
                    scheme.to_string()
                );
                println!("Secret Recovery Phrase : [{phrase}]");

                // Obtain account address
                let address = AccountAddress::from(new_address);

                // Create account action
                let action = AccountModule::create_account_action(address);

                let result = context
                    .sign_and_execute(new_address, action, scheme)
                    .await?;
                context.assert_execute_success(result)
            }
        }
    }
}
