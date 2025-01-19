// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod commands;

use crate::cli_types::CommandAction;
use crate::commands::da::commands::exec::ExecCommand;
use crate::commands::da::commands::index::IndexCommand;
use crate::commands::da::commands::namespace::NamespaceCommand;
use crate::commands::da::commands::unpack::UnpackCommand;
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
            DACommand::Unpack(unpack) => unpack.execute().map(|_| "".to_owned()),
            DACommand::Namespace(namespace) => namespace.execute().map(|_| "".to_owned()),
            DACommand::Exec(exec) => exec.execute().await.map(|_| "".to_owned()),
            DACommand::Index(index) => {
                index.execute()?;
                Ok("".to_owned())
            }
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "da")]
pub enum DACommand {
    Unpack(UnpackCommand),
    Namespace(NamespaceCommand),
    Exec(Box<ExecCommand>),
    Index(IndexCommand),
}
