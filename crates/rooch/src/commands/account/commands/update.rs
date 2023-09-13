// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    address::RoochAddress,
    chain_id::RoochChainID,
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
    /// Command line input of multichain ids
    #[clap(short = 'm', long = "multichain-id")]
    pub multichain_id: RoochChainID,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpdateCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        println!("{:?}", self.mnemonic_phrase);
        let mut context = self.context_options.build().await?;

        match self.multichain_id {
            RoochChainID::Builtin(_) => {
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
                    .update_address_with_key_pair_from_multichain_id(
                        &existing_address,
                        self.mnemonic_phrase,
                        self.multichain_id.clone(),
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
                    self.multichain_id.chain_id().id()
                );

                // Get public key
                let public_key = kp.public();

                // Get public key reference
                let public_key = public_key.as_ref().to_vec();

                // Create MoveAction from native validator
                let action = NativeValidatorModule::rotate_authentication_key_action(public_key);

                // Execute the Move call as a transaction
                let result = context
                    .sign_and_execute(existing_address, action, self.multichain_id)
                    .await?;
                context.assert_execute_success(result)
            }
            RoochChainID::Custom(_) => todo!(),
        }
    }
}
