// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::env::current_dir;

fn main() {
    if std::env::var("SKIP_STDLIB_BUILD").is_err() {
        let _ = tracing_subscriber::fmt::try_init();
        let current_dir = current_dir().expect("Should be able to get current dir");
        // Get the project root directory
        let mut root_dir = current_dir;
        root_dir.pop();
        root_dir.pop();

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/move-stdlib")
                .join("Move.toml")
                .display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/move-stdlib")
                .join("sources")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/moveos-stdlib")
                .join("Move.toml")
                .display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/moveos-stdlib")
                .join("sources")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/rooch-framework")
                .join("Move.toml")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/rooch-framework")
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
        match framework_builder::releaser::release_latest() {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "cargo::warning=\"Failed to release latest framework: {:?}\"",
                    e
                );
            }
        }
    }
}
