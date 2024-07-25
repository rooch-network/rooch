// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

fn main() {
    // Get the current version from the Cargo.toml
    let cargo_version = env!("CARGO_PKG_VERSION");

    // Prepend 'v' to the version
    let version_with_v = format!("v{}", cargo_version);

    // Get the commit hash for the version specified in CARGO_PKG_VERSION
    let git_commit_output = Command::new("git")
        .args(["rev-list", "-n", "1", &version_with_v])
        .output()
        .expect("Failed to get git commit hash for version");
    let git_commit_hash = String::from_utf8(git_commit_output.stdout)
        .expect("Invalid UTF-8 sequence")
        .trim()
        .to_string();

    // Set the environment variables
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_commit_hash);

    println!("cargo:rerun-if-changed=build.rs");
}
