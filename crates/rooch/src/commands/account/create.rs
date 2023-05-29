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
use moveos::moveos::TransactionOutput;
use moveos_types::{
    move_types::FunctionId,
    transaction::{MoveAction, MoveOSTransaction},
};
use rooch_client::Client;

use rooch_common::config::{
    rooch_config_dir, PersistedConfig, RoochConfig, ROOCH_CONFIG, ROOCH_KEYSTORE_FILENAME,
};
use rooch_key::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use rooch_types::{
    address::RoochAddress,
    cli::{CliError, CliResult},
    crypto::BuiltinScheme::Ed25519,
    transaction::{
        authenticator::Authenticator,
        rooch::{RoochTransaction, RoochTransactionData},
    },
};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateCommand {
    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

impl CreateCommand {
    pub async fn execute(
        self,
        config: &mut PersistedConfig<RoochConfig>,
    ) -> CliResult<TransactionOutput> {
        let (new_address, phrase, scheme) = config
            .keystore
            .generate_and_add_new_key(Ed25519, None, None)?;

        println!("{}", new_address.0);
        println!(
            "Generated new keypair for address with scheme {:?} [{new_address}]",
            scheme.to_string()
        );
        println!("Secret Recovery Phrase : [{phrase}]");
        //TODO define static variable.
        let create_account_entry_function =
            FunctionId::from_str("0x1::account::create_account_entry").unwrap();
        let action = MoveAction::new_function_call(
            create_account_entry_function,
            vec![],
            vec![bcs::to_bytes(&new_address).unwrap()],
        );

        // TODO: Code refactoring
        let sender: RoochAddress = new_address;
        let sequence_number = self.client.get_sequence_number(sender).await?;

        let tx = config
            .keystore
            .sign_transaction(
                &new_address,
                RoochTransactionData::new(sender, sequence_number, action),
            )
            .map_err(|e| CliError::SignMessageError(e.to_string()))?;

        self.client
            .execute_tx(tx)
            .await
            .map_err(|e| CliError::TransactionError(e.to_string()))
    }
}
