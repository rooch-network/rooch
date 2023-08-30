// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::env::current_dir;

fn main() {
    if std::env::var("SKIP_RPC_SPECK_BUILD").is_err() {
        let current_dir = current_dir().expect("Should be able to get current dir");
        // Get the project root directory
        let mut root_dir = current_dir;
        root_dir.pop();
        root_dir.pop();

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("crates/rooch-rpc-api")
                .join("Cargo.toml")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("crates/rooch-rpc-api")
                .join("src")
                .display()
        );
        rooch_open_rpc_spec_builder::build_and_save_rooch_rpc_spec().expect("build and save rooch rpc spec failed");
    }
}
