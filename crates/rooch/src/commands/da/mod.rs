// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod commands;

use crate::cli_types::CommandAction;
use crate::commands::da::commands::accumulator_anomaly::AccumulatorAnomalyCommand;
use crate::commands::da::commands::exec::ExecCommand;
use crate::commands::da::commands::index::IndexCommand;
use crate::commands::da::commands::namespace::NamespaceCommand;
use crate::commands::da::commands::pack::PackCommand;
use crate::commands::da::commands::repair::RepairCommand;
use crate::commands::da::commands::unpack::UnpackCommand;
use crate::commands::da::commands::verify::VerifyCommand;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

/// DA Commands
#[derive(Parser)]
pub struct DA {
    #[clap(subcommand)]
    cmd: DACommand,
}

#[async_trait]
impl CommandAction<String> for DA {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            DACommand::Pack(pack) => pack.execute().map(|_| "".to_owned()),
            DACommand::Unpack(unpack) => unpack.execute().map(|_| "".to_owned()),
            DACommand::Namespace(namespace) => namespace.execute().map(|_| "".to_owned()),
            DACommand::Exec(exec) => exec.execute().await.map(|_| "".to_owned()),
            DACommand::Index(index) => {
                index.execute().await?;
                Ok("".to_owned())
            }
            DACommand::Verify(verify) => {
                verify.execute().await?;
                Ok("".to_owned())
            }
            DACommand::Repair(repair) => {
                repair.execute().await?;
                Ok("".to_owned())
            }
            DACommand::AccumulatorAnomaly(accumulator_anomaly) => {
                accumulator_anomaly.execute().await?;
                Ok("".to_owned())
            }
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "da")]
pub enum DACommand {
    Pack(PackCommand),
    Unpack(UnpackCommand),
    Namespace(NamespaceCommand),
    Exec(Box<ExecCommand>),
    Index(IndexCommand),
    Verify(VerifyCommand),
    Repair(RepairCommand),
    AccumulatorAnomaly(AccumulatorAnomalyCommand),
}
