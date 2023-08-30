// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::ArgEnum;
use clap::Parser;
use pretty_assertions::assert_str_eq;
use std::fs::File;
use std::io::Write;


#[derive(Debug, Parser, Clone, Copy, ArgEnum)]
enum Action {
    Print,
    Test,
    Record,
}

#[derive(Debug, Parser)]
#[clap(
    name = "Rooch Open RPC Spec",
    about = "Generate rooch open rpc spec",
)]
struct Options {
    #[clap(arg_enum, default_value = "Record", ignore_case = true)]
    action: Action,
}


#[tokio::main]
async fn main() {
    let options = Options::parse();

    let open_rpc = rooch_open_rpc_spec_builder::build_rooch_rpc_spec();


    match options.action {
        Action::Record => {
            let content = serde_json::to_string_pretty(&open_rpc).unwrap();
            let mut f = File::create(rooch_open_rpc_spec_builder::spec_file()).unwrap();
            writeln!(f, "{content}").unwrap();
        }
        Action::Test => {
            let reference = std::fs::read_to_string(rooch_open_rpc_spec_builder::spec_file()).unwrap();
            let content = serde_json::to_string_pretty(&open_rpc).unwrap() + "\n";
            assert_str_eq!(&reference, &content);
        }
        _ => {
            let content = serde_json::to_string_pretty(&open_rpc).unwrap();
            println!("{content}");
        }
    }
}