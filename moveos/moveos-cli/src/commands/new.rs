// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_cli::base::new;
use move_core_types::account_address::AccountAddress;
use moveos_stdlib::addresses::{
    MOS_FRAMEWORK_ADDRESS, MOS_FRAMEWORK_ADDRESS_NAME, MOS_STD_ADDRESS, MOS_STD_ADDRESS_NAME,
};
use std::path::PathBuf;

const MOS_STDLIB_PKG_NAME: &str = "MosStdLib";
const MOS_STDLIB_PKG_PATH: &str = "{ git = \"https://github.com/rooch-network/moveos.git\", subdir = \"crates/moveos_stdlib/mos-stdlib\", rev = \"main\" }";

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
            //TODO add mos_framework as dependency.
            [(MOS_STDLIB_PKG_NAME, MOS_STDLIB_PKG_PATH)],
            [
                //TODO let dev pass the address as option.
                (name, &AccountAddress::random().to_hex_literal()),
                (
                    &MOS_STD_ADDRESS_NAME.to_string(),
                    &MOS_STD_ADDRESS.to_hex_literal(),
                ),
                (
                    &MOS_FRAMEWORK_ADDRESS_NAME.to_string(),
                    &MOS_FRAMEWORK_ADDRESS.to_hex_literal(),
                ),
            ],
            "",
        )?;
        Ok(())
    }
}
