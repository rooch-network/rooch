// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use clap::ArgEnum;
use clap::Parser;
use rooch_open_rpc::Project;
use rooch_rpc_api::api::rooch_api::RoochAPIOpenRpc;
use anyhow::Result;

mod examples;

pub fn rooch_rpc_doc(version: &str) -> Project {
    Project::new(
        version,
        "Rooch JSON-RPC",
        "Rooch JSON-RPC API for interaction with rooch server. ",
        "Rooch Network",
        "https://rooch.network",
        "opensource@rooch.network",
        "Apache-2.0",
        "https://raw.githubusercontent.com/rooch-network/rooch/main/LICENSE",
    )
}

#[derive(Debug, Parser, Clone, Copy, ArgEnum)]
enum Action {
    Print,
    Test,
    Record,
}
// TODO: This currently always use workspace version, which is not ideal.
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn build_rooch_rpc_spec() -> Project{
    let mut open_rpc = rooch_rpc_doc(VERSION);
    open_rpc.add_module(RoochAPIOpenRpc::module_doc()); 
    //open_rpc.add_examples(RpcExampleProvider::new().examples());
    open_rpc
}

pub fn build_and_save_rooch_rpc_spec() -> Result<()> {
    let open_rpc = build_rooch_rpc_spec();
    let content = serde_json::to_string_pretty(&open_rpc)?;
    let mut f = File::create(spec_file()).unwrap();
    writeln!(f, "{content}")?;
    Ok(())
}

pub fn spec_file() -> PathBuf{
    path_in_crate("../rooch-open-rpc-spec/schemas/openrpc.json")
}

fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: AsRef<Path>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative);
    path
}