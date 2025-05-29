// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use self::commands::create::CreateCommand;
use self::commands::keygen::KeygenCommand;
use self::commands::manage::ManageCommand;
use self::commands::query::QueryCommand;
use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;
use serde_json::Value;

pub mod commands;

#[derive(Parser)]
#[clap(about = "DID (Decentralized Identifier) management commands")]
pub struct DID {
    #[clap(subcommand)]
    cmd: DIDCommand,
}

#[async_trait]
impl CommandAction<String> for DID {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            DIDCommand::Create(create) => {
                let resp = create.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            DIDCommand::Manage(manage) => {
                let resp = manage.execute().await?;
                Ok(serde_json::to_string_pretty(&resp)?)
            }
            DIDCommand::Query(query) => {
                let json_output = query.execute_serialized().await?;
                let json_value: Value = serde_json::from_str(&json_output)?;

                // For now, just return JSON. Later we can add table formatting
                Ok(serde_json::to_string_pretty(&json_value)?)
            }
            DIDCommand::Keygen(keygen) => {
                let json_output = keygen.execute_serialized().await?;
                let json_value: Value = serde_json::from_str(&json_output)?;

                Ok(serde_json::to_string_pretty(&json_value)?)
            }
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "did")]
pub enum DIDCommand {
    /// Create a new DID
    #[clap(name = "create")]
    Create(CreateCommand),

    /// Manage DID (verification methods, services, etc.)
    #[clap(name = "manage")]
    Manage(ManageCommand),

    /// Query DID information
    #[clap(name = "query")]
    Query(QueryCommand),

    /// Generate cryptographic keys for DID operations
    #[clap(name = "keygen")]
    Keygen(KeygenCommand),
}
