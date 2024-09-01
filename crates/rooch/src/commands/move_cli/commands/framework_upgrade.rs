// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use framework_builder::Stdlib;
use move_core_types::account_address::AccountAddress;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use moveos_types::addresses::{
    MOVEOS_STD_ADDRESS, MOVEOS_STD_ADDRESS_NAME, MOVE_STD_ADDRESS, MOVE_STD_ADDRESS_NAME,
};
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::module_store::PackageData;
use moveos_types::{move_types::FunctionId, transaction::MoveAction};
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::addresses::{
    BITCOIN_MOVE_ADDRESS, BITCOIN_MOVE_ADDRESS_NAME, ROOCH_FRAMEWORK_ADDRESS,
    ROOCH_FRAMEWORK_ADDRESS_NAME,
};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::rooch::RoochTransaction;
use rpassword::prompt_password;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
pub struct FrameworkUpgrade {
    /// Path to a package which the command should be run with respect to.
    #[clap(long = "path", short = 'p', global = true, value_parser)]
    pub package_path: Option<PathBuf>,

    /// The address of the package to upgrade.
    /// Available options are: 0x1, 0x2, 0x3, 0x4
    #[clap(long)]
    pub package_id: AccountAddress,

    // TODO: remove this when reset genesis.
    /// This flag is used to upgrade the framework with old ABI: rooch_framework::upgrade::upgrade_entry
    #[clap(long, default_value = "false")]
    pub legacy: bool,

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

        let action = if self.legacy {
            println!("[Warning] upgrade with old abi: upgrade_entry");
            let args = vec![
                bcs::to_bytes(bundles_map.get(&MOVE_STD_ADDRESS).unwrap()).unwrap(),
                bcs::to_bytes(bundles_map.get(&MOVEOS_STD_ADDRESS).unwrap()).unwrap(),
                bcs::to_bytes(bundles_map.get(&ROOCH_FRAMEWORK_ADDRESS).unwrap()).unwrap(),
                bcs::to_bytes(bundles_map.get(&BITCOIN_MOVE_ADDRESS).unwrap()).unwrap(),
            ];
            MoveAction::new_function_call(
                FunctionId::new(
                    ModuleId::new(
                        ROOCH_FRAMEWORK_ADDRESS,
                        Identifier::new("upgrade".to_owned()).unwrap(),
                    ),
                    Identifier::new("upgrade_entry".to_owned()).unwrap(),
                ),
                vec![],
                args,
            )
        } else {
            let pkg_id = self.package_id;
            let pkg_name = match pkg_id {
                MOVE_STD_ADDRESS => MOVE_STD_ADDRESS_NAME,
                MOVEOS_STD_ADDRESS => MOVEOS_STD_ADDRESS_NAME,
                ROOCH_FRAMEWORK_ADDRESS => ROOCH_FRAMEWORK_ADDRESS_NAME,
                BITCOIN_MOVE_ADDRESS => BITCOIN_MOVE_ADDRESS_NAME,
                _ => {
                    return Err(RoochError::CommandArgumentError(
                        "Invalid package id".to_owned(),
                    ))
                }
            };
            let package_data = PackageData::new(
                MoveString::from(pkg_name),
                pkg_id,
                bundles_map.get(&pkg_id).unwrap().clone(),
            );
            let pkg_bytes = bcs::to_bytes(&package_data).unwrap();
            MoveAction::new_function_call(
                FunctionId::new(
                    ModuleId::new(
                        MOVEOS_STD_ADDRESS,
                        Identifier::new("module_store".to_owned()).unwrap(),
                    ),
                    Identifier::new("publish_package_entry".to_owned()).unwrap(),
                ),
                vec![],
                vec![bcs::to_bytes(&pkg_bytes).unwrap()],
            )
        };

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
