// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use clap::Parser;
use move_cli::base::new;
use move_core_types::account_address::AccountAddress;
use moveos_types::addresses::{
    MOVEOS_STD_ADDRESS, MOVEOS_STD_ADDRESS_NAME, MOVE_STD_ADDRESS, MOVE_STD_ADDRESS_NAME,
};
use rooch_config::ROOCH_CLIENT_CONFIG;
use rooch_framework::{ROOCH_FRAMEWORK_ADDRESS, ROOCH_FRAMEWORK_ADDRESS_NAME};
use rooch_types::error::RoochError;
use std::path::PathBuf;

use crate::cli_types::WalletContextOptions;

const MOVE_STDLIB_PKG_NAME: &str = "MoveStdlib";
const MOVE_STDLIB_PKG_PATH: &str = "{ git = \"https://github.com/rooch-network/rooch.git\", subdir = \"frameworks/move-stdlib\", rev = \"main\" }";

const MOVEOS_STDLIB_PKG_NAME: &str = "MoveosStdlib";
const MOVEOS_STDLIB_PKG_PATH: &str = "{ git = \"https://github.com/rooch-network/rooch.git\", subdir = \"frameworks/moveos-stdlib\", rev = \"main\" }";

const ROOCH_FRAMEWORK_PKG_NAME: &str = "RoochFramework";
const ROOCH_FRAMEWORK_PKG_PATH: &str = "{ git = \"https://github.com/rooch-network/rooch.git\", subdir = \"frameworks/rooch-framework\", rev = \"main\" }";

#[derive(Parser)]
pub struct New {
    /// Existing account address from Rooch
    #[clap(long = "address", short = 'a')]
    account_address: Option<AccountAddress>,

    ///The name of the package to be created.
    name: String,

    #[clap(flatten)]
    wallet_context_options: WalletContextOptions,
}

impl New {
    async fn get_active_account_address_from_config(&self) -> Result<String, RoochError> {
        // build wallet context options
        let context = self.wallet_context_options.build()?;
        // get active account address value
        match context.client_config.active_address {
            Some(address) => Ok(AccountAddress::from(address).to_hex_literal()),
            None => Err(RoochError::ConfigLoadError(
                ROOCH_CLIENT_CONFIG.to_string(),
                format!(
                    "No active address found in {}. Check if {} is complete",
                    ROOCH_CLIENT_CONFIG, ROOCH_CLIENT_CONFIG,
                ),
            )),
        }
    }

    pub async fn execute(self, path: Option<PathBuf>) -> anyhow::Result<()> {
        let name = &self.name.to_lowercase();
        let address = if let Some(account_address) = &self.account_address {
            // Existing account address is available
            account_address.to_hex_literal()
        } else {
            // Existing account address is not available, use the active address from config file generated from the command `rooch init`
            match self.get_active_account_address_from_config().await {
                Ok(active_account_address) => active_account_address,
                Err(err) => return Err(anyhow!("{}", err)),
            }
        };

        let new_cli = new::New {
            name: name.to_string(),
        };
        new_cli.execute(
            path,
            "0.0.1",
            [
                (MOVE_STDLIB_PKG_NAME, MOVE_STDLIB_PKG_PATH),
                (MOVEOS_STDLIB_PKG_NAME, MOVEOS_STDLIB_PKG_PATH),
                (ROOCH_FRAMEWORK_PKG_NAME, ROOCH_FRAMEWORK_PKG_PATH),
            ],
            [
                (name, &address),
                (
                    &MOVE_STD_ADDRESS_NAME.to_string(),
                    &MOVE_STD_ADDRESS.to_hex_literal(),
                ),
                (
                    &MOVEOS_STD_ADDRESS_NAME.to_string(),
                    &MOVEOS_STD_ADDRESS.to_hex_literal(),
                ),
                (
                    &ROOCH_FRAMEWORK_ADDRESS_NAME.to_string(),
                    &ROOCH_FRAMEWORK_ADDRESS.to_hex_literal(),
                ),
            ],
            "",
        )
    }
}
