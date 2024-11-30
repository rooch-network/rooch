//! A build script generating rust types from protobuf definitions.

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use prost_types::FileDescriptorSet;
use std::collections::HashSet;

static OUT_DIR: &str = "src/proto";
const PROTO_FILES: &[&str] = &[
    "src/proto/finalitygadget.proto",
];

const INCLUDES: &[&str] = &["src/proto"];

fn main() {
    let fds = protox_compile();
    // #[cfg(not(feature = "tonic"))]
    // prost_build(fds);
    // #[cfg(feature = "tonic")]
    tonic_build(fds)
}

fn protox_compile() -> FileDescriptorSet {
    protox::compile(PROTO_FILES, INCLUDES).expect("protox failed to build")
}

// #[cfg(not(feature = "tonic"))]
fn prost_build(fds: FileDescriptorSet) {
    let mut config = prost_build::Config::new();

    config
        // .include_file("mod.rs")
        .enable_type_names()
        .out_dir(OUT_DIR)
        // .bytes([".tendermint_celestia_mods.abci"])
        .compile_fds(fds)
        .expect("prost failed");
}

// #[cfg(feature = "tonic")]
fn tonic_build(fds: FileDescriptorSet) {
    let mut prost_config = prost_build::Config::new();
    prost_config.enable_type_names();

    let mut tonic_config = tonic_build::configure()
        // .include_file("mod.rs")
        .build_client(true)
        .build_server(false)
        .out_dir(OUT_DIR)
        // .client_mod_attribute(".", "#[cfg(not(target_arch=\"wasm32\"))]")
        // .use_arc_self(true)
        .compile_well_known_types(true)
        .skip_protoc_run();
        // .bytes([".tendermint_celestia_mods.abci"]);

    tonic_config
        .compile_fds_with_config(prost_config, fds)
        .expect("should be able to compile protobuf using tonic");
}


