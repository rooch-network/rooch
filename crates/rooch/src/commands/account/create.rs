// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]
use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::parse_type_tag,
};
use moveos::moveos::TransactionOutput;
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use rooch_client::Client;

use rooch_common::config::{
    rooch_config_dir, PersistedConfig, RoochConfig, ROOCH_CONFIG, ROOCH_KEYSTORE_FILENAME,
};
use rooch_key::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use rooch_server::response::JsonResponse;
use rooch_types::{
    account::SignatureScheme::ED25519,
    address::RoochAddress,
    cli::{CliError, CliResult},
    transaction::{authenticator::AccountPrivateKey, rooch::RoochTransactionData},
};
use std::path::{Path, PathBuf};

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
    ) -> CliResult<JsonResponse<TransactionOutput>> {
        let (new_address, phrase, scheme) = config
            .keystore
            .generate_and_add_new_key(ED25519, None, None)
            .map_err(|e| CliError::GenerateKeyError(e.to_string()))?;

        println!("{}", new_address.0);
        println!(
            "Generated new keypair for address with scheme {:?} [{new_address}]",
            scheme.to_string()
        );
        println!("Secret Recovery Phrase : [{phrase}]");

        let action = MoveAction::new_function(
            ModuleId::new(AccountAddress::ONE, ident_str!("account").to_owned()),
            ident_str!("create_account_entry").to_owned(),
            vec![],
            vec![bcs::to_bytes(&new_address).unwrap()],
        );

        let sender: RoochAddress = new_address;
        let sequence_number = self.client.get_sequence_number(sender).await?;
        let tx_data = RoochTransactionData::new(sender, sequence_number, action);
        //TODO sign the tx by the account private key
        let private_key = AccountPrivateKey::generate_for_testing();
        let tx = tx_data.sign(&private_key)?;

        self.client
            .submit_txn(tx)
            .await
            .map_err(|e| CliError::TransactionError(e.to_string()))
    }
}
