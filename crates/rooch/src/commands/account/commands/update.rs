// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hex::ToHex;
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
    /// Whether a password should be provided.
    #[clap(short = 'p', long = "password")]
    password_required: bool,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}
#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpdateCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        println!("{:?}", self.mnemonic_phrase);

        let password = if self.password_required {
            // Prompt for a password if required
            rpassword::prompt_password("Enter a password to encrypt the keys in the rooch keystore. Press return to have an empty value: ").unwrap()
        } else {
            // Use an empty password if not required
            String::new()
        };
        println!("Your password is {}", password);

        let mut context = self.context_options.build().await?;

        let existing_address = RoochAddress::from_str(&self.address).map_err(|e| {
            RoochError::CommandArgumentError(format!("Invalid Rooch address String: {}", e))
        })?;

        let password = rpassword::prompt_password("Enter a password to encrypt the keys in rooch keystore. Empty password leaves an unencrypted key: ").unwrap();
        println!("Your password is {}", password);

        let result = context
            .keystore
            .update_address_with_key_pair_from_key_pair_type(
                &existing_address,
                self.mnemonic_phrase,
                KeyPairType::RoochKeyPairType,
                None,
                Some(password),
            )
            .map_err(|e| RoochError::UpdateAccountError(e.to_string()))?;

        context.config.password = Some(result.encryption.hashed_password);
        context.config.nonce = Some(result.encryption.nonce.encode_hex());
        context.config.ciphertext = Some(result.encryption.ciphertext.encode_hex());
        context.config.tag = Some(result.encryption.tag.encode_hex());
        context.config.save()?;

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
        let public_key = result.key_pair.public();

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
