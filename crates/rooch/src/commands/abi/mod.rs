// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod commands;

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use commands::{export::ExportCommand};
use rooch_types::error::{RoochError, RoochResult};
use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct ABI {
    #[clap(subcommand)]
    cmd: ABICommand,
    /// Sets the file storing the state of our user accounts (an empty one will be created if missing)
    #[clap(long = "client.config")]
    config: Option<PathBuf>,
    #[clap(short = 'y', long = "yes")]
    accept_defaults: bool,
}

#[async_trait]
impl CommandAction<String> for ABI {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            ABICommand::Export(export) => export.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
        }
        .map_err(RoochError::from)
    }
}

#[derive(Debug, clap::Subcommand)]
#[clap(name = "abi")]
pub enum ABICommand {
    Export(ExportCommand),
}