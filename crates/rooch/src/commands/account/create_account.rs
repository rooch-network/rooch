// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]
use anyhow::{Ok, Result};
use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::parse_type_tag,
};
use moveos_client::Client;
use moveos_types::transaction::{MoveTransaction, SimpleTransaction};

use crate::config::{rooch_config_dir, ROOCH_CONFIG, ROOCH_KEYSTORE_FILENAME};
use rooch_key::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use rooch_types::account::SignatureScheme::ED25519;
use std::path::{Path, PathBuf};

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateAccount {
    // /// Address of the new account
    // #[clap(long)]
    // pub address: AccountAddress,

    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

impl CreateAccount {
    pub async fn execute(self) -> Result<()> {
        let config_path = rooch_config_dir()?.join(ROOCH_CONFIG);
        prompt_if_no_config(&config_path).await?;

        // TODO:
        // let txn = MoveTransaction::new_function(
        //     ModuleId::new(
        //         AccountAddress::new([
        //             0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //             0, 0, 0, 0, 0, 1,
        //         ]),
        //         ident_str!("account").to_owned(),
        //     ),
        //     ident_str!("create_account_entry").to_owned(),
        //     vec![],
        //     vec![bcs::to_bytes(&self.address).unwrap()],
        // );
        // let txn = SimpleTransaction::new(self.address, txn);

        // self.client.submit_txn(txn).await?;
        Ok(())
    }
}

// Prompt user for connect to devnet fullnode if config does not exist.
async fn prompt_if_no_config(conf_path: &Path) -> Result<(), anyhow::Error> {
    let keystore_path = conf_path
        .parent()
        .unwrap_or(&rooch_config_dir()?)
        .join(ROOCH_KEYSTORE_FILENAME);
    let mut keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);

    let (new_address, phrase, scheme) =
        keystore.generate_and_add_new_key(ED25519, None, None)?;
    println!(
        "Generated new keypair for address with scheme {:?} [{new_address}]",
        scheme.to_string()
    );
    println!("Secret Recovery Phrase : [{phrase}]");

    Ok(())
}
