// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use moveos_server::{Execute, OsServer};

#[tokio::main]
async fn main() {
    let args = OsServer::parse();

    let _result = args.command.execute().await;

    // TODO: handle error.
}
