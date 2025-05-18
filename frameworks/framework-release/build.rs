// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::env::current_dir;

fn main() {
    // Skip the release process on Windows to avoid stack overflow
    if cfg!(windows) {
        println!("cargo:warning=Skipping framework release on Windows to avoid stack overflow");
        println!("cargo:warning=The framework will be built on non-Windows platforms");
        return;
    }

    if std::env::var("SKIP_STDLIB_BUILD").is_err() {
        std::env::set_var("RUST_LOG", "WARN");
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

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/rooch-nursery")
                .join("Move.toml")
                .display()
        );

        println!(
            "cargo:rerun-if-changed={}",
            root_dir
                .join("frameworks/rooch-nursery")
                .join("sources")
                .display()
        );

        match framework_builder::releaser::release_latest() {
            Ok(msgs) => {
                for msg in msgs {
                    println!("cargo::warning=\"{}\"", msg);
                }
            }
            Err(e) => {
                println!(
                    "cargo::warning=\"Failed to release latest framework: {:?}\"",
                    e
                );
            }
        }
    }
}
