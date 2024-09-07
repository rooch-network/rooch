// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::{Parser, Subcommand};
use commands::{
    build::BuildCommand, coverage::CoverageCommand, disassemble::DisassembleCommand,
    docgen::DocgenCommand, errmap::ErrmapCommand, info::InfoCommand,
    integration_test::IntegrationTestCommand, new::NewCommand, prove::ProveCommand,
    publish::Publish, run_function::RunFunction, run_view_function::RunViewFunction,
    unit_test::TestCommand,
};
use rooch_types::error::RoochResult;
use serde_json::{json, Value};

use crate::commands::move_cli::commands::explain::ExplainCommand;
use crate::CommandAction;

pub mod commands;

#[derive(Parser)]
pub struct MoveCli {
    #[clap(subcommand)]
    cmd: MoveCommand,
}

#[derive(Subcommand)]
#[clap(name = "move")]
pub enum MoveCommand {
    Build(BuildCommand),
    Coverage(CoverageCommand),
    Disassemble(DisassembleCommand),
    Docgen(DocgenCommand),
    Errmap(ErrmapCommand),
    Info(InfoCommand),
    New(NewCommand),
    Prove(ProveCommand),
    Test(TestCommand),
    Publish(Publish),
    Run(RunFunction),
    View(RunViewFunction),
    IntegrationTest(IntegrationTestCommand),
    Explain(ExplainCommand),
}

#[async_trait]
impl CommandAction<String> for MoveCli {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            MoveCommand::Build(c) => c.execute_serialized().await,
            MoveCommand::Coverage(c) => c.execute_serialized().await,
            MoveCommand::Disassemble(c) => c.execute_serialized().await,
            MoveCommand::Docgen(c) => c.execute_serialized().await,
            MoveCommand::Errmap(c) => c.execute_serialized().await,
            MoveCommand::Info(c) => c.execute_serialized().await,
            MoveCommand::New(c) => c.execute_serialized().await,
            MoveCommand::Prove(c) => c.execute_serialized().await,
            MoveCommand::Test(c) => c.execute_serialized().await,
            MoveCommand::Publish(c) => c.execute_serialized().await,
            MoveCommand::Run(c) => c.execute_serialized().await,
            MoveCommand::View(c) => c.execute_serialized().await,
            MoveCommand::IntegrationTest(c) => c.execute_serialized().await,
            MoveCommand::Explain(c) => c.execute_serialized().await,
        }
    }
}

pub fn serialized_success(json: bool) -> RoochResult<Option<Value>> {
    if json {
        let json_result = json!({ "Result": "Success" });
        Ok(Some(json_result))
    } else {
        Ok(None)
    }
}

pub fn print_serialized_success(json: bool) -> RoochResult<Option<Value>> {
    if json {
        let json_result = json!({ "Result": "Success" });
        Ok(Some(json_result))
    } else {
        println!("Success");
        Ok(None)
    }
}
