// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{error::RoochResult, keypair_type::KeyPairType};

/// Create a new account off-chain.
/// If an account not exist on-chain, contract will auto create the account on-chain.
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateCommand {
    /// Whether a password should be provided
    #[clap(long = "password")]
    password_required: Option<bool>,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let mut context = self.context_options.build().await?;

        let password = if self.password_required == Some(false) {
            // Use an empty password if not required
            String::new()
        } else {
            // Prompt for a password if required
            rpassword::prompt_password("Enter a password to encrypt the keys in the rooch keystore. Press return to have an empty value: ").unwrap()
        };
        println!("Your password is {}", password);

        let result = context.keystore.generate_and_add_new_key(
            KeyPairType::RoochKeyPairType,
            None,
            None,
            Some(password),
        )?;

        let address = AccountAddress::from(result.address).to_hex_literal();
        println!(
            "Generated new keypair for address with key pair type {:?} [{}]",
            result.result.key_pair_type, result.address
        );
        println!("Secret Recovery Phrase : [{}]", result.result.mnemonic);

        Ok(address)
    }
}
