// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]
use anyhow::Result;
use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    effects::Op,
    ident_str,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::parse_type_tag,
};
use moveos_types::transaction::TransactionOutput;
use moveos_types::{
    move_types::FunctionId,
    transaction::{MoveAction, MoveOSTransaction},
};
use once_cell::sync::Lazy;
use rooch_framework::ROOCH_FRAMEWORK_ADDRESS;
use rooch_key::keystore::{AccountKeystore, Keystore};
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::{
    address::RoochAddress,
    crypto::BuiltinScheme,
    error::{RoochError, RoochResult},
    transaction::{
        authenticator::Authenticator,
        rooch::{RoochTransaction, RoochTransactionData},
    },
};

use crate::cli_types::{CommandAction, WalletContextOptions};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

/// Update an address with a new keypair from scheme to rooch.keystore
#[derive(Debug, Parser)]
pub struct UpdateCommand {
    /// Rooch address in string format.
    #[clap(short = 'a', long = "address")]
    address: String,
    #[clap(short = 'm', long = "mnemonic-phrase")]
    mnemonic_phrase: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// Command line input of crypto schemes (ed25519, multied25519, ecdsa, or schnorr)
    #[clap(short = 's', long = "scheme", arg_enum)]
    pub crypto_schemes: BuiltinScheme,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpdateCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        println!("{:?}", self.mnemonic_phrase);

        let mut context = self.context_options.build().await?;

        match BuiltinScheme::from_flag_byte(&self.crypto_schemes.flag()) {
            Ok(scheme) => {
                let existing_address =
                    RoochAddress::from_str(self.address.as_str()).map_err(|e| {
                        RoochError::CommandArgumentError(format!(
                            "Invalid Rooch address String: {}",
                            e
                        ))
                    })?;

                context
                .config
                .keystore
                .update_address_with_key_pair_from_scheme(
                    &existing_address,
                    self.mnemonic_phrase,
                    scheme,
                    None,
                )
                .map_err(|e| RoochError::UpdateAccountError(e.to_string()))?;

                println!(
                    "{}",
                    AccountAddress::from(existing_address).to_hex_literal()
                );
                println!(
                    "Generated a new keypair for an existing address on scheme {:?} [{existing_address}]",
                    scheme.to_owned()
                );

                // TODO leave room to handle it if it's not from a built-in scheme,
                // install a custom validator to the account using update account function in Move
                // and verify the custom signature scheme.
                let update_account_entry_function = UPDATE_ACCOUNT_ENTRY_FUNCTION.clone();
                let action = MoveAction::new_function_call(
                    update_account_entry_function,
                    vec![],
                    vec![bcs::to_bytes(&existing_address).unwrap()],
                );

                context
                    .sign_and_execute(existing_address, action, scheme)
                    .await
            }
            Err(error) => {
                return Err(RoochError::CommandArgumentError(format!(
                    "Invalid crypto scheme: {}",
                    error
                )))
            }
        }
    }
}

static UPDATE_ACCOUNT_ENTRY_FUNCTION: Lazy<FunctionId> = Lazy::new(|| {
    FunctionId::new(
        ModuleId::new(ROOCH_FRAMEWORK_ADDRESS, Identifier::new("account").unwrap()),
        Identifier::new("update_account_entry").unwrap(),
    )
});
