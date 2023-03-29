// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;

#[derive(clap::Parser)]
#[clap(author, version, about, long_about = None)]
pub struct RoochCli {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(clap::Parser)]
pub enum Command {
    Move(moveos_cli::MoveCli),
}

pub fn run_cli(opt: RoochCli) -> Result<()> {
    match opt.cmd {
        Command::Move(move_cli) => moveos_cli::run_cli(move_cli),
    }
}
