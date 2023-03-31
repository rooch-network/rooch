// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_cli::base::new;
use move_core_types::account_address::AccountAddress;
use moveos_stdlib::addresses::{
    MOVEOS_STD_ADDRESS, MOVEOS_STD_ADDRESS_NAME, ROOCH_FRAMEWORK_ADDRESS,
    ROOCH_FRAMEWORK_ADDRESS_NAME,
};
use std::path::PathBuf;

//TODO allow external packages to be added as dependencies, add rooch-framework as dependency.
const MOVEOS_STDLIB_PKG_NAME: &str = "MoveosStdLib";
const MOVEOS_STDLIB_PKG_PATH: &str = "{ git = \"https://github.com/rooch-network/rooch.git\", subdir = \"moveos/moveos-stdlib/moveos-stdlib\", rev = \"main\" }";

#[derive(Parser)]
pub struct New {
    #[clap(flatten)]
    pub new: new::New,
}

impl New {
    pub fn execute(self, path: Option<PathBuf>) -> anyhow::Result<()> {
        let name = &self.new.name.to_lowercase();
        self.new.execute(
            path,
            "0.0.1",
            //TODO add rooch_framework as dependency.
            [(MOVEOS_STDLIB_PKG_NAME, MOVEOS_STDLIB_PKG_PATH)],
            [
                //TODO let dev pass the address as option.
                (name, &AccountAddress::random().to_hex_literal()),
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
        )?;
        Ok(())
    }
}
