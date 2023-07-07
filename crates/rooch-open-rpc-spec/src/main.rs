// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::ArgEnum;
use clap::Parser;
use pretty_assertions::assert_str_eq;
use rooch_open_rpc::Project;
use rooch_rpc_api::api::rooch_api::RoochAPIOpenRpc;
use std::fs::File;
use std::io::Write;

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

#[derive(Debug, Parser)]
#[clap(
    name = "Sui format generator",
    about = "Trace serde (de)serialization to generate format descriptions for Sui types"
)]
struct Options {
    #[clap(arg_enum, default_value = "Record", ignore_case = true)]
    action: Action,
}

const FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/schemas/openrpc.json",);

// TODO: This currently always use workspace version, which is not ideal.
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let options = Options::parse();

    let mut open_rpc = rooch_rpc_doc(VERSION);
    open_rpc.add_module(RoochAPIOpenRpc::module_doc());

    // open_rpc.add_examples(RpcExampleProvider::new().examples());

    match options.action {
        Action::Record => {
            let content = serde_json::to_string_pretty(&open_rpc).unwrap();
            let mut f = File::create(FILE_PATH).unwrap();
            writeln!(f, "{content}").unwrap();
        }
        Action::Test => {
            let reference = std::fs::read_to_string(FILE_PATH).unwrap();
            let content = serde_json::to_string_pretty(&open_rpc).unwrap() + "\n";
            assert_str_eq!(&reference, &content);
        }
        _ => {
            let content = serde_json::to_string_pretty(&open_rpc).unwrap();
            println!("{content}");
        }
    }
}

#[test]
fn test_json_rpc_spec() {
    // If this test breaks and you intended a json rpc schema change, you need to run to get the fresh schema:
    // # cargo run --package rooch-open-rpc-spec --bin rooch-open-rpc-spec -- record
    let status = std::process::Command::new("cargo")
        .args([
            "run",
            "--package",
            "rooch-open-rpc-spec",
            "--bin",
            "rooch-open-rpc-spec",
            "--",
        ])
        .arg("test")
        .status()
        .expect("msg");

    assert!(
        status.success(),
            "\n\
    If this test breaks and you intended a json rpc schema change, you need to run to get the fresh schema:\n\
    ./scripts/generate_json_rpc_spec.sh
            "
        );
}
