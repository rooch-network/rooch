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
use moveos_types::transaction::TransactionOutput;
use moveos_types::{
    move_types::FunctionId,
    transaction::{MoveAction, MoveOSTransaction},
};
use rooch_client::wallet_context::WalletContext;
use rooch_key::keystore::{AccountKeystore, Keystore};
use rooch_server::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    address::RoochAddress,
    crypto::BuiltinScheme::Ed25519,
    error::{RoochError, RoochResult},
    transaction::{
        authenticator::Authenticator,
        rooch::{RoochTransaction, RoochTransactionData},
    },
};

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::types::{CommandAction, WalletContextOptions};

/// Create a new account on-chain
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
    pub async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        let mut context = self.context_options.build().await?;
        let (new_address, phrase, scheme) = context
            .config
            .keystore
            .generate_and_add_new_key(Ed25519, None, None)?;

        println!("{}", AccountAddress::from(new_address).to_hex_literal());
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

        context.sign_and_execute(new_address, action).await
    }
}
