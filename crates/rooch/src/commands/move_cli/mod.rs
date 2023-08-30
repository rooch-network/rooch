// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use commands::{
    build::Build, integration_test::IntegrationTest, new::New, publish::Publish,
    run_function::RunFunction, run_view_function::RunViewFunction, unit_test::Test,
};
use move_cli::{
    base::{
        coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap, info::Info,
        prove::Prove,
    },
    Move,
};
use rooch_types::error::{RoochError, RoochResult};

use crate::commands::move_cli::commands::explain::Explain;
use crate::CommandAction;

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
    View(RunViewFunction),
    IntegrationTest(IntegrationTest),
    Explain(Explain),
}

#[async_trait]
impl CommandAction<String> for MoveCli {
    async fn execute(self) -> RoochResult<String> {
        let move_args = self.move_args;
        match self.cmd {
            MoveCommand::Build(c) => c
                .execute(move_args.package_path, move_args.build_config)
                .await
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::Coverage(c) => c
                .execute(move_args.package_path, move_args.build_config)
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::Disassemble(c) => c
                .execute(move_args.package_path, move_args.build_config)
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::Docgen(c) => c
                .execute(move_args.package_path, move_args.build_config)
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::Errmap(mut c) => {
                c.error_prefix = Some("Error".to_owned());
                c.execute(move_args.package_path, move_args.build_config)
                    .map(|_| "Success".to_owned())
                    .map_err(RoochError::from)
            }
            MoveCommand::Info(c) => c
                .execute(move_args.package_path, move_args.build_config)
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::New(c) => c
                .execute(move_args.package_path)
                .await
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::Prove(c) => c
                .execute(move_args.package_path, move_args.build_config)
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::Test(c) => c
                .execute(move_args.package_path, move_args.build_config)
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::Publish(c) => c.execute_serialized().await,
            MoveCommand::Run(c) => c.execute_serialized().await,
            MoveCommand::View(c) => c.execute_serialized().await,
            MoveCommand::IntegrationTest(c) => c
                .execute(move_args)
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
            MoveCommand::Explain(c) => c
                .execute()
                .await
                .map(|_| "Success".to_owned())
                .map_err(RoochError::from),
        }
    }
}
