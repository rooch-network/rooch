// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use commands::new::New;
use framework::natives::{all_natives, GasParameters};
use move_cli::{
    base::{
        build::Build, coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap,
        info::Info, prove::Prove, test::Test,
    },
    Move,
};

pub mod commands;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CliOptions {
    #[clap(flatten)]
    pub move_args: Move,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser)]
pub enum Command {
    Build(Build),
    Coverage(Coverage),
    Disassemble(Disassemble),
    Docgen(Docgen),
    Errmap(Errmap),
    Info(Info),
    New(New),
    Prove(Prove),
    Test(Test),
    //TODO implement run command
    //TODO implement integration test command
}

pub fn run_cli(move_args: Move, cmd: Command) -> Result<()> {
    //let error_descriptions: ErrorMapping = bcs::from_bytes(framework::error_descriptions())?;
    //TODO define gas metering
    let cost_table = move_vm_test_utils::gas_schedule::INITIAL_COST_SCHEDULE.clone();
    let natives = all_natives(GasParameters::zeros());

    match cmd {
        Command::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Coverage(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Disassemble(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Docgen(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Errmap(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Info(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::New(c) => c.execute(move_args.package_path),
        Command::Prove(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Test(c) => c.execute(
            move_args.package_path,
            move_args.build_config,
            natives,
            Some(cost_table),
        ),
    }
}
