// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use rooch::RoochCli;

/// rooch is a command line tools for Rooch Network
fn main() -> Result<()> {
    let opt = RoochCli::parse();
    rooch::run_cli(opt).unwrap();
    Ok(())
}
