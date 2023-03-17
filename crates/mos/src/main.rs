// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use mos::CliOptions;

/// mos is a command line tools for MoveOS
fn main() -> Result<()> {
    let opt = CliOptions::parse();
    mos::run_cli(opt.move_args, opt.cmd).unwrap();
    Ok(())
}
