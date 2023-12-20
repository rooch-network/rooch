// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::env::current_dir;

fn main() {
    if std::env::var("SKIP_STDLIB_BUILD").is_err() {
        let current_dir = current_dir().expect("Should be able to get current dir");
        // Get the project root directory
        let mut root_dir = current_dir;
        root_dir.pop();
        root_dir.pop();

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("moveos/moveos-stdlib/move-stdlib")
                .join("Move.toml")
                .display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("moveos/moveos-stdlib/move-stdlib")
                .join("sources")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("moveos/moveos-stdlib/moveos-stdlib")
                .join("Move.toml")
                .display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("moveos/moveos-stdlib/moveos-stdlib")
                .join("sources")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("crates/rooch-framework")
                .join("Move.toml")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("crates/rooch-framework")
                .join("sources")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/bitcoin-move")
                .join("Move.toml")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/bitcoin-move")
                .join("sources")
                .display()
        );
        rooch_genesis_builder::build_and_save_stdlib().expect("Build stdlib failed")
    }
}
