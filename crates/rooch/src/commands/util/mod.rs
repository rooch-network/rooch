// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::CommandAction;
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use commands::{address::AddressCommand, hex::HexCommand};
use rooch_types::error::RoochResult;

pub mod commands;

#[derive(Parser)]
pub struct Util {
    #[clap(subcommand)]
    cmd: UtilCommand,
}

#[derive(Subcommand)]
#[clap(name = "util")]
pub enum UtilCommand {
    Hex(HexCommand),
    Address(AddressCommand),
}

#[async_trait]
impl CommandAction<String> for Util {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            UtilCommand::Hex(c) => c.execute().await,
            UtilCommand::Address(c) => c.execute_serialized().await,
        }
    }
}
