// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use commands::{
    new::New, publish::Publish, run_function::RunFunction, run_view_function::RunViewFunction,
    unit_test::Test,
};
use move_cli::{
    base::{
        build::Build, coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap,
        info::Info, prove::Prove,
    },
    Move,
};
use rooch_types::cli::{CliError, CliResult};

pub mod commands;
pub mod types;

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
    View(RunViewFunction),
    //TODO implement integration test command
}

pub async fn run_cli(move_cli: MoveCli) -> CliResult<()> {
    let move_args = move_cli.move_args;
    let cmd = move_cli.cmd;
    //let error_descriptions: ErrorMapping = bcs::from_bytes(moveos_stdlib::error_descriptions())?;

    match cmd {
        MoveCommand::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Coverage(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Disassemble(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Docgen(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Errmap(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Info(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::New(c) => c.execute(move_args.package_path),
        MoveCommand::Prove(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Test(c) => c.execute(move_args.package_path, move_args.build_config),
        MoveCommand::Publish(c) => {
            c.execute(move_args.package_path, move_args.build_config)
                .await
        }
        MoveCommand::Run(c) => c.execute().await,
        MoveCommand::View(c) => c.execute().await,
    }
    .map_err(CliError::from)
}
