// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::CommandAction;
use async_trait::async_trait;
use commands::{
    create::CreateCommand, import::ImportCommand, list::ListCommand, object::ObjectCommand,
    resource::ResourceCommand,
};
use rooch_types::error::{RoochError, RoochResult};
use std::path::PathBuf;

pub mod commands;

#[derive(clap::Parser)]
pub struct Account {
    #[clap(subcommand)]
    cmd: AccountCommand,
    /// Sets the file storing the state of our user accounts (an empty one will be created if missing)
    #[clap(long = "client.config")]
    config: Option<PathBuf>,
    #[clap(short = 'y', long = "yes")]
    accept_defaults: bool,
}

#[async_trait]
impl CommandAction<String> for Account {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            AccountCommand::Create(create) => create.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            AccountCommand::List(list) => list.execute().await.map(|_| "".to_string()),
            AccountCommand::Import(import) => import.execute().await.map(|_| "".to_string()),
            AccountCommand::Resource(resource) => resource.execute_serialized().await,
            AccountCommand::Object(object) => object.execute_serialized().await,
        }
        .map_err(RoochError::from)
    }
}

#[derive(Debug, clap::Subcommand)]
#[clap(name = "account")]
pub enum AccountCommand {
    Create(CreateCommand),
    List(ListCommand),
    Import(ImportCommand),
    Object(ObjectCommand),
    Resource(ResourceCommand),
}
