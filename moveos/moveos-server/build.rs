// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

fn main() {
    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");

    tonic_build::configure()
        .out_dir("src/pb")
        .compile_with_config(config, &["protos/server.proto"], &["protos"])
        .unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=protos");
}
