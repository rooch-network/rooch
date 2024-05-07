// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;

use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use rooch_key::key_derive::verify_password;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::transaction::rooch::RoochTransaction;
use rpassword::prompt_password;

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use framework_builder::Stdlib;
use moveos_types::addresses::{MOVEOS_STD_ADDRESS, MOVE_STD_ADDRESS};
use moveos_types::{move_types::FunctionId, transaction::MoveAction};
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::addresses::{BITCOIN_MOVE_ADDRESS, ROOCH_FRAMEWORK_ADDRESS};
use rooch_types::error::{RoochError, RoochResult};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
pub struct FrameworkUpgrade {
    /// Path to a package which the command should be run with respect to.
    #[clap(long = "path", short = 'p', global = true, value_parser)]
    pub package_path: Option<PathBuf>,

    #[clap(flatten)]
    context_options: WalletContextOptions,

    #[clap(flatten)]
    tx_options: TransactionOptions,
}
#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for FrameworkUpgrade {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        // Clone variables for later use
        let package_path = self
            .package_path
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        let context = self
            .context_options
            .build()
            .expect("Building context failed.");

        let stdlib = Stdlib::load_from_file(package_path)?;
        let bundles_map: HashMap<_, _> = stdlib
            .all_module_bundles()
            .expect("get bundles failed")
            .into_iter()
            .collect();
        let args = vec![
            bcs::to_bytes(bundles_map.get(&MOVE_STD_ADDRESS).unwrap()).unwrap(),
            bcs::to_bytes(bundles_map.get(&MOVEOS_STD_ADDRESS).unwrap()).unwrap(),
            bcs::to_bytes(bundles_map.get(&ROOCH_FRAMEWORK_ADDRESS).unwrap()).unwrap(),
            bcs::to_bytes(bundles_map.get(&BITCOIN_MOVE_ADDRESS).unwrap()).unwrap(),
        ];
        let action = MoveAction::new_function_call(
            FunctionId::new(
                ModuleId::new(
                    ROOCH_FRAMEWORK_ADDRESS,
                    Identifier::new("upgrade".to_owned()).unwrap(),
                ),
                Identifier::new("upgrade_entry".to_owned()).unwrap(),
            ),
            vec![],
            args,
        );

        // Build context and handle errors
        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Handle transaction with or without authenticator
        match self.tx_options.authenticator {
            Some(authenticator) => {
                let tx_data = context
                    .build_tx_data(sender, action, max_gas_amount)
                    .await?;
                let tx = RoochTransaction::new(tx_data, authenticator.into());
                context.execute(tx).await
            }
            None => {
                if context.keystore.get_if_password_is_empty() {
                    context
                        .sign_and_execute(sender, action, None, max_gas_amount)
                        .await
                } else {
                    let password =
                        prompt_password("Enter the password to publish:").unwrap_or_default();
                    let is_verified = verify_password(
                        Some(password.clone()),
                        context.keystore.get_password_hash(),
                    )?;

                    if !is_verified {
                        return Err(RoochError::InvalidPasswordError(
                            "Password is invalid".to_owned(),
                        ));
                    }

                    context
                        .sign_and_execute(sender, action, Some(password), max_gas_amount)
                        .await
                }
            }
        }
    }
}
