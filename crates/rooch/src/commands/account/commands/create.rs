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

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// Command line input of crypto schemes (0 for Ed25519, 1 for MultiEd25519, 2 for Ecdsa, 3 for Schnorr)
    #[clap(short = 's', long = "scheme")]
    pub crypto_schemes: String,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let mut context = self.context_options.build().await?;
        match BuiltinScheme::from_flag(self.crypto_schemes.clone().trim()) {
            Ok(scheme) => {
                let (new_address, phrase, scheme) = context
                    .config
                    .keystore
                    .generate_and_add_new_key(scheme, None, None)?;

                println!("{}", AccountAddress::from(new_address).to_hex_literal());
                println!(
                    "Generated new keypair for address with scheme {:?} [{new_address}]",
                    scheme.to_string()
                );
                println!("Secret Recovery Phrase : [{phrase}]");

                let create_account_entry_function = CREATE_ACCOUNT_ENTRY_FUNCTION.clone();
                let action = MoveAction::new_function_call(
                    create_account_entry_function,
                    vec![],
                    vec![bcs::to_bytes(&new_address).unwrap()],
                );

                context.sign_and_execute(new_address, action).await
            }
            Err(error) => Err(RoochError::CommandArgumentError(format!(
                "Invalid crypto scheme: {}",
                error
            ))),
        }
    }
}

static CREATE_ACCOUNT_ENTRY_FUNCTION: Lazy<FunctionId> = Lazy::new(|| {
    FunctionId::new(
        ModuleId::new(ROOCH_FRAMEWORK_ADDRESS, Identifier::new("account").unwrap()),
        Identifier::new("create_account_entry").unwrap(),
    )
});
