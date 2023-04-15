// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use moveos_server::{Execute, OsServer};

#[tokio::main]
async fn main() -> Result<()> {
    let args = OsServer::parse();

    args.command.execute().await
}
