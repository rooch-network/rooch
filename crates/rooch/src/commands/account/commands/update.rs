// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::{keypair::KeyPairType, keystore::AccountKeystore};
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    address::RoochAddress,
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
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpdateCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        println!("{:?}", self.mnemonic_phrase);
        let mut context = self.context_options.build().await?;

        let existing_address = RoochAddress::from_str(self.address.as_str()).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
        })?;

        let kp = context
            .client_config
            .keystore
            .update_address_with_key_pair_from_key_pair_type(
                &existing_address,
                self.mnemonic_phrase,
                KeyPairType::RoochKeyPairType,
                None,
            )
            .map_err(|e| RoochError::UpdateAccountError(e.to_string()))?;

        println!(
            "{}",
            AccountAddress::from(existing_address).to_hex_literal()
        );
        println!(
            "Generated a new keypair for an existing address {:?} for type {:?}",
            existing_address,
            KeyPairType::RoochKeyPairType.type_of()
        );

        // Get public key
        let public_key = kp.public();

        // Get public key reference
        let public_key = public_key.as_ref().to_vec();

        // Create MoveAction from native validator
        let action = NativeValidatorModule::rotate_authentication_key_action(public_key);

        // Execute the Move call as a transaction
        let result = context
            .sign_and_execute(existing_address, action, KeyPairType::RoochKeyPairType)
            .await?;
        context.assert_execute_success(result)
    }
}
