// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use rooch_types::error::RoochResult;

pub mod commands;
pub mod generator;
pub mod inscribe;
pub mod inscription;
pub mod operation;
pub mod sft;

pub const PROTOCOL: &str = "bitseed";
pub const METADATA_OP: &str = "op";
pub const METADATA_TICK: &str = "tick";
pub const METADATA_AMOUNT: &str = "amount";
pub const METADATA_ATTRIBUTES: &str = "attributes";
pub const GENERATOR_TICK: &str = "generator";

/// Tool for interacting with bitseed protocol
#[derive(clap::Parser)]
pub struct Bitseed {
    #[clap(subcommand)]
    cmd: BitseedCommand,
}

#[async_trait]
impl CommandAction<String> for Bitseed {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            BitseedCommand::Generator(generator) => generator.execute_serialized().await,
            BitseedCommand::Deploy(deploy) => deploy.execute_serialized().await,
            BitseedCommand::Mint(mint) => mint.execute_serialized().await,
            BitseedCommand::Split(split) => split.execute_serialized().await,
            BitseedCommand::Merge(merge) => merge.execute_serialized().await,
            BitseedCommand::View(view) => view.execute_serialized().await,
        }
    }
}

#[derive(Debug, clap::Subcommand)]
#[clap(name = "bitseed")]
pub enum BitseedCommand {
    Generator(commands::generator::GeneratorCommand),
    Deploy(commands::deploy::DeployCommand),
    Mint(commands::mint::MintCommand),
    Split(commands::split::SplitCommand),
    Merge(commands::merge::MergeCommand),
    View(commands::view::ViewCommand),
}
