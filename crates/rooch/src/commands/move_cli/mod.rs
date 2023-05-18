// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use commands::{
    build::Build, new::New, publish::Publish, run_function::RunFunction,
    run_view_function::RunViewFunction, unit_test::Test,
};
use move_cli::{
    base::{
        coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap, info::Info,
        prove::Prove,
    },
    Move,
};
use rooch_types::cli::{CliError, CliResult, CommandAction};

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

impl MoveCli {
    pub async fn execute(self) -> CliResult<String> {
        run_cli(self).await
    }
}

async fn run_cli(move_cli: MoveCli) -> CliResult<String> {
    let move_args = move_cli.move_args;
    let cmd = move_cli.cmd;

    //let error_descriptions: ErrorMapping = bcs::from_bytes(moveos_stdlib::error_descriptions())?;

    match cmd {
        MoveCommand::Build(c) => c
            .execute(move_args.package_path, move_args.build_config)
            .map(|_| "Success".to_string())
            .map_err(CliError::from),
        MoveCommand::Coverage(c) => c
            .execute(move_args.package_path, move_args.build_config)
            .map(|_| "Success".to_string())
            .map_err(CliError::from),
        MoveCommand::Disassemble(c) => c
            .execute(move_args.package_path, move_args.build_config)
            .map(|_| "Success".to_string())
            .map_err(CliError::from),
        MoveCommand::Docgen(c) => c
            .execute(move_args.package_path, move_args.build_config)
            .map(|_| "Success".to_string())
            .map_err(CliError::from),
        MoveCommand::Errmap(c) => c
            .execute(move_args.package_path, move_args.build_config)
            .map(|_| "Success".to_string())
            .map_err(CliError::from),
        MoveCommand::Info(c) => c
            .execute(move_args.package_path, move_args.build_config)
            .map(|_| "Success".to_string())
            .map_err(CliError::from),
        MoveCommand::New(c) => c.execute_serialized().await,
        MoveCommand::Prove(c) => c
            .execute(move_args.package_path, move_args.build_config)
            .map(|_| "Success".to_string())
            .map_err(CliError::from),
        MoveCommand::Test(c) => c
            .execute(move_args.package_path, move_args.build_config)
            .map(|_| "Success".to_string())
            .map_err(CliError::from),
        MoveCommand::Publish(c) => c.execute_serialized().await,
        MoveCommand::Run(c) => c.execute_serialized().await,
        MoveCommand::View(c) => c.execute_serialized().await,
    }
}
