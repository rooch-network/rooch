// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::MoveStructType;
use rooch_rpc_client::Client;
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::did::{DIDDocument, DIDModule};
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
    /// Query DID document by DID string
    #[clap(name = "did")]
    ByDID(ByDIDCommand),

    /// Query DID document by ObjectID
    #[clap(name = "object-id")]
    ByObjectID(ByObjectIDCommand),

    /// Query DID document by address
    #[clap(name = "address")]
    ByAddress(ByAddressCommand),

    /// Query DID documents controlled by a specific controller
    #[clap(name = "controller")]
    ByController(ByControllerCommand),

    /// Check if a DID document exists
    #[clap(name = "exists")]
    Exists(ExistsCommand),
}

#[derive(Debug, Parser)]
pub struct ByDIDCommand {
    /// DID identifier string
    #[clap(help = "DID identifier (e.g., did:rooch:bc1q... or did:key:z6MkpTHR8VNs...)")]
    pub did: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct ByObjectIDCommand {
    /// Object ID of the DID document
    #[clap(help = "Object ID of the DID document")]
    pub object_id: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct ByAddressCommand {
    /// Rooch address
    #[clap(help = "Rooch address (e.g., 0x123... or bc1q...)")]
    pub address: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct ByControllerCommand {
    /// Controller DID string
    #[clap(help = "Controller DID string (e.g., did:key:z6MkpTHR8VNs... or did:rooch:bc1q...)")]
    pub controller_did: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct ExistsCommand {
    /// DID identifier string or Rooch address to check
    #[clap(help = "DID identifier or Rooch address (e.g., did:rooch:bc1q... or bc1q...)")]
    pub identifier: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DIDDocumentOutput {
    pub did_document: DIDDocument,
    pub object_id: ObjectID,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlledDIDsOutput {
    pub controller: String,
    pub controlled_dids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExistsOutput {
    pub identifier: String,
    pub exists: bool,
    pub query_type: String,
}

#[async_trait]
impl CommandAction<serde_json::Value> for QueryCommand {
    async fn execute(self) -> RoochResult<serde_json::Value> {
        match self.query_type {
            QueryType::ByDID(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
            QueryType::ByObjectID(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
            QueryType::ByAddress(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
            QueryType::ByController(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
            QueryType::Exists(cmd) => {
                let result = cmd.execute().await?;
                Ok(serde_json::to_value(result)?)
            }
        }
    }
}

impl QueryCommand {
    pub async fn execute_serialized(self) -> RoochResult<String> {
        let result = self.execute().await?;
        Ok(serde_json::to_string(&result)?)
    }
}

#[async_trait]
impl CommandAction<DIDDocumentOutput> for ByDIDCommand {
    async fn execute(self) -> RoochResult<DIDDocumentOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;

        // Parse DID string to extract identifier
        if !self.did.starts_with("did:") {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Invalid DID format, must start with 'did:'".to_string(),
            ));
        }

        let parts: Vec<&str> = self.did.split(':').collect();
        if parts.len() < 3 {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Invalid DID format".to_string(),
            ));
        }
        let method = parts[1];

        if method != "rooch" {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Only rooch DID method is supported for document retrieval".to_string(),
            ));
        }

        let identifier_part = parts[2..].join(":");

        //Check if the identifier is a Rooch address
        let rooch_address = RoochAddress::from_str(&identifier_part)?;
        let did_identifier = rooch_address.to_bech32();

        let object_id = moveos_types::moveos_std::object::custom_object_id(
            &did_identifier,
            &DIDDocument::struct_tag(),
        );

        let did_document_output = get_did_document_by_object_id(&client, object_id).await?;

        Ok(did_document_output)
    }
}

#[async_trait]
impl CommandAction<DIDDocumentOutput> for ByObjectIDCommand {
    async fn execute(self) -> RoochResult<DIDDocumentOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;

        let object_id = ObjectID::from_str(&self.object_id).map_err(|e| {
            rooch_types::error::RoochError::CommandArgumentError(format!(
                "Invalid object ID: {}",
                e
            ))
        })?;

        let did_document_output = get_did_document_by_object_id(&client, object_id).await?;

        Ok(did_document_output)
    }
}

#[async_trait]
impl CommandAction<DIDDocumentOutput> for ByAddressCommand {
    async fn execute(self) -> RoochResult<DIDDocumentOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;

        let address = RoochAddress::from_str(&self.address)?;

        let did_identifier = address.to_bech32();
        let object_id = moveos_types::moveos_std::object::custom_object_id(
            &did_identifier,
            &DIDDocument::struct_tag(),
        );

        let did_document_output = get_did_document_by_object_id(&client, object_id).await?;

        Ok(did_document_output)
    }
}

#[async_trait]
impl CommandAction<ControlledDIDsOutput> for ByControllerCommand {
    async fn execute(self) -> RoochResult<ControlledDIDsOutput> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let did_module = client.as_module_binding::<DIDModule>();

        // Validate DID format
        if !self.controller_did.starts_with("did:") {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Controller must be a valid DID string".to_string(),
            ));
        }

        let dids = did_module.get_dids_by_controller_string(&self.controller_did)?;

        Ok(ControlledDIDsOutput {
            controller: self.controller_did,
            controlled_dids: dids,
        })
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
            let address = RoochAddress::from_str(&self.identifier)?;
            let exists = did_module.exists_did_for_address(address.into())?;
            (exists, "rooch_address".to_string())
        };

        Ok(ExistsOutput {
            identifier: self.identifier,
            exists,
            query_type,
        })
    }
}

async fn get_did_document_by_object_id(
    client: &Client,
    object_id: ObjectID,
) -> RoochResult<DIDDocumentOutput> {
    let mut did_object_views = client
        .rooch
        .get_object_states(vec![object_id.clone()], None)
        .await?;
    if did_object_views.is_empty() || did_object_views.first().unwrap().is_none() {
        return Err(rooch_types::error::RoochError::CommandArgumentError(
            format!("DID document with object ID {} not found", object_id),
        ));
    }
    let did_object_view = did_object_views.pop().unwrap().unwrap();
    let did_document = bcs::from_bytes::<DIDDocument>(&did_object_view.value.0).map_err(|_| {
        rooch_types::error::RoochError::CommandArgumentError(format!(
            "Failed to deserialize DID document with object ID {}",
            object_id
        ))
    })?;
    Ok(DIDDocumentOutput {
        did_document,
        object_id,
        created_at: did_object_view.metadata.created_at.0,
        updated_at: did_object_view.metadata.updated_at.0,
    })
}
