// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::env;

fn main() {
    // Compile the probestack_fix.c file to fix linking issues with wasmer on Rust 1.91
    // See: https://github.com/rust-lang/rust/issues/142612
    let target = env::var("TARGET").unwrap();
    if target.contains("x86_64") {
        cc::Build::new()
            .file("probestack_fix.c")
            .compile("probestack_fix");

        println!("cargo:rerun-if-changed=probestack_fix.c");
    }
}
