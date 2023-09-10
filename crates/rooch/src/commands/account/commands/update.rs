// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKeyType,
    coin_type::CoinID,
    error::{RoochError, RoochResult},
    framework::native_validator::NativeValidatorModule,
};

use crate::cli_types::{CommandAction, WalletContextOptions};
use std::str::FromStr;

/// Update an address with a new keypair from coin id to rooch.keystore
#[derive(Debug, Parser)]
pub struct UpdateCommand {
    /// Rooch address in string format.
    #[clap(short = 'a', long = "address")]
    address: String,
    #[clap(short = 'm', long = "mnemonic-phrase")]
    mnemonic_phrase: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// Authentication key type. Select an authentication key type with the Rooch address (Bitcoin P2PKH authentication key address leading with "1" or Bitcoin P2SH authentication key address leading with "3")
    #[clap(short = 't', long = "authentication-key-type", arg_enum)]
    pub authentication_key_type: Option<AuthenticationKeyType>,
    /// Command line input of coin ids
    #[clap(short = 'c', long = "coin-id", arg_enum)]
    pub coin_id: CoinID,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpdateCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        println!("{:?}", self.mnemonic_phrase);

        let mut context = self.context_options.rooch_build().await?;

        match self.coin_id {
            CoinID::Rooch => {
                let existing_address =
                    RoochAddress::from_str(self.address.as_str()).map_err(|e| {
                        RoochError::CommandArgumentError(format!(
                            "Invalid Rooch address String: {}",
                            e
                        ))
                    })?;

                let kp = context
                    .config
                    .keystore
                    .update_address_with_key_pair_from_coin_id(
                        &existing_address,
                        self.mnemonic_phrase,
                        self.coin_id,
                        None,
                    )
                    .map_err(|e| RoochError::UpdateAccountError(e.to_string()))?;

                println!(
                    "{}",
                    AccountAddress::from(existing_address).to_hex_literal()
                );
                println!(
                    "Generated a new keypair for an existing address {:?} on coin id {:?}",
                    existing_address,
                    self.coin_id.to_owned()
                );

                // Get public key
                let public_key = kp.public();

                // Get public key reference
                let public_key = public_key.as_ref().to_vec();

                // Create MoveAction from native validator
                let action = NativeValidatorModule::rotate_authentication_key_action(public_key);

                // Execute the Move call as a transaction
                let result = context
                    .sign_and_execute(existing_address, action, self.coin_id)
                    .await?;
                context.assert_execute_success(result)
            }
            CoinID::Bitcoin => todo!(),
            CoinID::Ether => todo!(),
            // CoinID::Ether => {
            // let existing_address =
            //     EthereumAddress::from_str(self.address.as_str()).map_err(|e| {
            //         RoochError::CommandArgumentError(format!(
            //             "Invalid Ethereum address String: {}",
            //             e
            //         ))
            //     })?;

            // let kp = context
            //     .config
            //     .keystore
            //     .update_address_with_key_pair_from_coin_id(
            //         &existing_address,
            //         self.mnemonic_phrase,
            //         self.coin_id,
            //         None,
            //     )
            //     .map_err(|e| RoochError::UpdateAccountError(e.to_string()))?;

            // println!(
            //     "{:?}",
            //     AccountAddress::from_hex_literal(&existing_address.0.to_string())
            // );
            // println!(
            //     "Generated a new keypair for an existing address {:?} on coin id {:?}",
            //     existing_address,
            //     self.coin_id.to_owned()
            // );

            // // Get public key
            // let public_key = kp.public();

            // // Get public key reference
            // let public_key = public_key.as_ref().to_vec();

            // // Create MoveAction from native validator
            // let action = NativeValidatorModule::rotate_authentication_key_action(public_key);

            // // Execute the Move call as a transaction
            // let result = context
            //     .sign_and_execute(existing_address, action, self.coin_id)
            //     .await?;
            // context.assert_execute_success(result)
            // }
            CoinID::Nostr => todo!(),
        }
    }
}
