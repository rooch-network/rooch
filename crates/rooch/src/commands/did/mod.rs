// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use commands::create::CreateCommand;
use commands::init::InitCommand;
use commands::query::QueryCommand;
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
            DIDCommand::Init(init) => init.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DIDCommand::Create(create) => create.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response") 
            }),
            DIDCommand::Query(query) => {
                let json_output = query.execute_serialized().await?;
                let json_value: Value = 
                    serde_json::from_str(&json_output).expect("Failed to parse JSON");
                
                // For now, just return JSON. Later we can add table formatting
                Ok(serde_json::to_string_pretty(&json_value).unwrap())
            }
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "did")]
pub enum DIDCommand {
    /// Initialize the DID registry
    #[clap(name = "init")]
    Init(InitCommand),
    
    /// Create a new DID
    #[clap(name = "create")]
    Create(CreateCommand),
    
    /// Query DID information
    #[clap(name = "query")]
    Query(QueryCommand),
} 