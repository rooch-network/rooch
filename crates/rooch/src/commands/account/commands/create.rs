// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use clap::Parser;
use hex::ToHex;
use move_core_types::account_address::AccountAddress;
use rooch_key::{keypair::KeyPairType, keystore::AccountKeystore};
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
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let mut context = self.context_options.build().await?;

        let password = rpassword::prompt_password("Enter a password to encrypt the keys in rooch keystore. Empty password leaves an unencrypted key: ").unwrap();
        println!("Your password is {}", password);

        let result = context.keystore.generate_and_add_new_key(
            KeyPairType::RoochKeyPairType,
            None,
            None,
            Some(password),
        )?;

        context.config.password = Some(result.result.encryption.hashed_password);
        context.config.nonce = Some(result.result.encryption.nonce.encode_hex());
        context.config.ciphertext = Some(result.result.encryption.ciphertext.encode_hex());
        context.config.tag = Some(result.result.encryption.tag.encode_hex());
        context.config.save()?;

        let address = AccountAddress::from(new_address).to_hex_literal();
        println!(
            "Generated new keypair for address with multichain id {:?} [{new_address}]",
            multichain_id
        );
        println!("Secret Recovery Phrase : [{}]", result.result.mnemonic);

        Ok(address)
    }
}
