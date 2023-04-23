// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use commands::{new::New, publish::Publish, run_function::RunFunction};
use move_cli::{
    base::{
        build::Build, coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap,
        info::Info, prove::Prove, test::Test,
    },
    Move,
};
use moveos_stdlib::natives::{all_natives, GasParameters};

pub mod commands;

#[derive(clap::Parser)]
pub struct MoveCli {
    #[clap(flatten)]
    move_args: Move,
    #[clap(subcommand)]
    cmd: MoveCommand,
}

#[derive(clap::Subcommand)]
#[clap(name = "move")]
pub enum MoveCommand {
    Build(Build),
    Coverage(Coverage),
    Disassemble(Disassemble),
    Docgen(Docgen),
    Errmap(Errmap),
    Info(Info),
    New(New),
    Prove(Prove),
    Test(Test),
    Publish(Publish),
    Run(RunFunction),
    //TODO implement run command
    //TODO implement integration test command
}

pub async fn run_cli(move_cli: MoveCli) -> Result<()> {
    let move_args = move_cli.move_args;
    let cmd = move_cli.cmd;
    //let error_descriptions: ErrorMapping = bcs::from_bytes(moveos_stdlib::error_descriptions())?;
    //TODO define gas metering
    let cost_table = move_vm_test_utils::gas_schedule::INITIAL_COST_SCHEDULE.clone();
    let natives = all_natives(GasParameters::zeros());

    match cmd {
        MoveCommand::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Coverage(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Disassemble(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Docgen(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Errmap(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Info(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::New(c) => c.execute(move_args.package_path),
        MoveCommand::Prove(c) => c.execute(move_args.package_path, move_args.build_config),
        //TODO how to custom the unit test and pass MoveOS to test plan
        //https://github.com/move-language/move/blob/dfef14a838c8dd6399e933b77426a03f40e69c4c/language/tools/move-unit-test/src/test_runner.rs#L286
        MoveCommand::Test(c) => c.execute(
            move_args.package_path,
            move_args.build_config,
            natives,
            Some(cost_table),
        ),
        MoveCommand::Publish(c) => {
            c.execute(move_args.package_path, move_args.build_config)
                .await
        }
        MoveCommand::Run(c) => c.execute().await,
    }
}
