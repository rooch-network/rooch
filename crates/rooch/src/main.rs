// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch::RoochCli;
use std::process::exit;

/// rooch is a command line tools for Rooch Network
#[tokio::main]
async fn main() {
    let opt = RoochCli::parse();
    let result = rooch::run_cli(opt).await;

    match result {
        Ok(s) => println!("{:?}", s),
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    }
}
