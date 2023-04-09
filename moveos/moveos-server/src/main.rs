// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use moveos_server::{say_hello, start_server, Command, OsServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = OsServer::parse();

    use Command::*;
    match args.command {
        Say(say) => say_hello(say).await?,
        Start(_) => start_server().await?,
    }

    Ok(())
}
