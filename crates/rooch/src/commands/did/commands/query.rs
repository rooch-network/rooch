// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::did::DIDModule;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Query DID information
#[derive(Debug, Parser)]
pub struct QueryCommand {
    #[clap(subcommand)]
    pub query_type: QueryType,
}

#[derive(Debug, Parser)]
pub enum QueryType {
    /// Check if a DID document exists
    #[clap(name = "exists")]
    Exists(ExistsCommand),
    
    /// Query DID documents controlled by a specific controller
    #[clap(name = "controlled")]
    Controlled(ControlledCommand),
}

#[derive(Debug, Parser)]
pub struct ExistsCommand {
    /// DID identifier string or Rooch address to check
    #[clap(help = "DID identifier or Rooch address (e.g., did:rooch:bc1q... or bc1q...)")]
    pub identifier: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct ControlledCommand {
    /// Controller DID string
    #[clap(help = "Controller DID string (e.g., did:key:z6MkpTHR8VNs... or did:rooch:bc1q...)")]
    pub controller_did: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExistsOutput {
    pub identifier: String,
    pub exists: bool,
    pub query_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlledOutput {
    pub controller: String,
    pub controlled_dids: Vec<ControlledDID>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlledDID {
    pub object_id: String,
}

#[async_trait]
impl CommandAction<serde_json::Value> for QueryCommand {
    async fn execute(self) -> RoochResult<serde_json::Value> {
        match self.query_type {
            QueryType::Exists(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
            QueryType::Controlled(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
        }
    }
}

#[async_trait]
impl CommandAction<ExistsOutput> for ExistsCommand {
    async fn execute(self) -> RoochResult<ExistsOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let did_module = client.as_module_binding::<DIDModule>();

        let (exists, query_type) = if self.identifier.starts_with("did:") {
            // Extract identifier part from DID string
            let parts: Vec<&str> = self.identifier.split(':').collect();
            if parts.len() >= 3 {
                let identifier_part = parts[2..].join(":");
                let exists = did_module.exists_did_document_by_identifier(&identifier_part)?;
                (exists, "did_identifier".to_string())
            } else {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    "Invalid DID format".to_string(),
                ));
            }
        } else {
            // Try to parse as Rooch address
            let address = if self.identifier.starts_with("0x") {
                AccountAddress::from_str(&self.identifier).map_err(|e| {
                    rooch_types::error::RoochError::CommandArgumentError(format!("Invalid address: {}", e))
                })?
            } else {
                // Try to parse as bech32
                let rooch_addr = RoochAddress::from_str(&self.identifier)?;
                rooch_addr.into()
            };
            let exists = did_module.exists_did_for_address(address)?;
            (exists, "rooch_address".to_string())
        };

        Ok(ExistsOutput {
            identifier: self.identifier,
            exists,
            query_type,
        })
    }
}

#[async_trait]
impl CommandAction<ControlledOutput> for ControlledCommand {
    async fn execute(self) -> RoochResult<ControlledOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let did_module = client.as_module_binding::<DIDModule>();

        // Validate DID format
        if !self.controller_did.starts_with("did:") {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Controller must be a valid DID string".to_string(),
            ));
        }

        let object_ids = did_module.get_dids_by_controller_string(&self.controller_did)?;
        
        let controlled_dids = object_ids
            .into_iter()
            .map(|id| ControlledDID {
                object_id: id.to_hex(),
            })
            .collect();

        Ok(ControlledOutput {
            controller: self.controller_did,
            controlled_dids,
        })
    }
} 