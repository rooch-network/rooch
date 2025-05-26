// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::MoveType;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::{OpView, StateChangeSetView, TransactionExecutionInfoView};
use rooch_types::error::RoochResult;
use rooch_types::framework::did::{DIDModule, DID};
use rooch_types::transaction::authenticator::SessionAuthenticator;
use rooch_types::transaction::RoochTransaction;
use rooch_types::{address::RoochAddress, framework::did::DIDDocument};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Create a new DID
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(subcommand)]
    pub create_type: CreateType,
}

#[derive(Debug, Parser)]
pub enum CreateType {
    /// Create a DID for yourself using your account key
    #[clap(name = "self")]
    SelfCreate(SelfCreateCommand),

    /// Create a DID via CADOP (Custodian-Assisted DID Onboarding Protocol)
    #[clap(name = "cadop")]
    Cadop(CadopCreateCommand),
}

#[derive(Debug, Parser)]
pub struct SelfCreateCommand {
    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Parser)]
pub struct CadopCreateCommand {
    /// User's did:key string
    #[clap(long, help = "User's did:key string (e.g., did:key:z6MkpTHR8VNs...)")]
    pub user_did_key: String,

    /// Custodian's service public key
    #[clap(long, help = "Custodian's service public key in multibase format")]
    pub custodian_service_key: String,

    /// Custodian's service verification method type
    #[clap(
        long,
        default_value = "Ed25519VerificationKey2020",
        help = "Custodian service VM type: Ed25519VerificationKey2020 or Secp256k1VerificationKey2019"
    )]
    pub custodian_key_type: String,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOutput {
    pub did: String,
    pub object_id: ObjectID,
    pub did_address: RoochAddress,
    pub execution_info: TransactionExecutionInfoView,
}

#[async_trait]
impl CommandAction<CreateOutput> for CreateCommand {
    async fn execute(self) -> RoochResult<CreateOutput> {
        match self.create_type {
            CreateType::SelfCreate(cmd) => cmd.execute().await,
            CreateType::Cadop(cmd) => cmd.execute().await,
        }
    }
}

#[async_trait]
impl CommandAction<CreateOutput> for SelfCreateCommand {
    async fn execute(self) -> RoochResult<CreateOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Get the public key from the wallet context
        let keypair = context.get_key_pair(&sender)?;
        let public_key = keypair.public();

        // Convert public key to multibase format (raw bytes without flag)
        let public_key_multibase_str = public_key.raw_to_multibase();
        let public_key_multibase = MoveString::from_str(&public_key_multibase_str)?;

        // Create the DID creation action
        let action = DIDModule::create_did_object_for_self_action(public_key_multibase);

        // Execute the transaction
        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;
        let result = context.sign_and_execute(sender, tx_data).await?;
        context.assert_execute_success(result.clone())?;

        // Extract the DID object ID from the changeset
        let output = result.output.clone().ok_or_else(|| {
            rooch_types::error::RoochError::CommandArgumentError(
                "Transaction output not available".to_string(),
            )
        })?;
        let did_object_id = extract_new_did_object_id(&output.changeset)?;

        let client = context.get_client().await?;
        let did_module = client.as_module_binding::<DIDModule>();

        let did_document = did_module.get_did_document_by_object_id(did_object_id.clone())?;
        // Calculate the DID identifier
        let did_identifier = did_document.id.to_string();

        let did_address = did_document.account_cap.addr.into();
        Ok(CreateOutput {
            did: did_identifier,
            object_id: did_object_id,
            did_address,
            execution_info: result.execution_info,
        })
    }
}

#[async_trait]
impl CommandAction<CreateOutput> for CadopCreateCommand {
    async fn execute(self) -> RoochResult<CreateOutput> {
        let context = self.context_options.build_require_password()?;
        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Validate did:key format
        if !self.user_did_key.starts_with("did:key:") {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "User DID must be in did:key format".to_string(),
            ));
        }

        let client = context.get_client().await?;
        let did_module = client.as_module_binding::<DIDModule>();

        let did_document = did_module.get_did_document(sender.into())?;
        let controllers = did_document.controller;
        if controllers.is_empty() {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                format!("DID {} has no controllers", sender),
            ));
        }
        let controller_did_struct: DID = controllers[0].clone();
        let controller_address = RoochAddress::from_str(controller_did_struct.identifier.as_str())?;

        if !context.keystore.contains_address(&controller_address) {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                format!(
                    "Keystore does not contain key for controller {}",
                    controller_address
                ),
            ));
        }

        // Create the CADOP DID creation action
        let user_did_key_string = MoveString::from_str(&self.user_did_key)?;
        let custodian_service_pk_multibase = MoveString::from_str(&self.custodian_service_key)?;
        let custodian_service_vm_type = MoveString::from_str(&self.custodian_key_type)?;

        let action = DIDModule::create_did_object_via_cadop_with_did_key_action(
            user_did_key_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type,
        );

        // Execute the transaction
        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;

        // Sign transaction with controller's key
        let kp = context.get_key_pair(&controller_address)?;
        let authenticator = SessionAuthenticator::sign(&kp, &tx_data);
        let tx = RoochTransaction::new(tx_data, authenticator.into());

        let result = context.execute(tx).await?;
        context.assert_execute_success(result.clone())?;

        // For CADOP, extract the new DID account address from changeset
        let output = result.output.clone().ok_or_else(|| {
            rooch_types::error::RoochError::CommandArgumentError(
                "Transaction output not available".to_string(),
            )
        })?;

        let client = context.get_client().await?;
        let did_module = client.as_module_binding::<DIDModule>();

        // Extract the DID object ID from the changeset
        let did_object_id = extract_new_did_object_id(&output.changeset)?;
        let did_document = did_module.get_did_document_by_object_id(did_object_id.clone())?;
        let did_identifier = did_document.id.to_string();
        let did_address = did_document.account_cap.addr.into();

        Ok(CreateOutput {
            did: did_identifier,
            object_id: did_object_id,
            did_address,
            execution_info: result.execution_info,
        })
    }
}

/// Extract the newly created DID object ID from the changeset
fn extract_new_did_object_id(
    changeset: &StateChangeSetView,
) -> RoochResult<moveos_types::moveos_std::object::ObjectID> {
    // Look for this object ID in the changeset
    for object_change in &changeset.changes {
        if let Some(op) = &object_change.value {
            if matches!(op, OpView::New(_))
                && object_change.metadata.object_type.0 == DIDDocument::type_tag()
            {
                return Ok(object_change.metadata.id.clone());
            }
        }
    }

    Err(rooch_types::error::RoochError::CommandArgumentError(
        "Failed to find newly created DID object in changeset".to_string(),
    ))
}
