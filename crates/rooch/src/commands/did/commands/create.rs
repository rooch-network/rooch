// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::move_std::string::MoveString;
use rooch_rpc_api::jsonrpc_types::{StateChangeSetView, OpView};
use rooch_types::address::RoochAddress;
use rooch_types::error::RoochResult;
use rooch_types::framework::did::DIDModule;
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
    
    /// User's public key (consistent with did:key)
    #[clap(long, help = "User's public key in multibase format")]
    pub user_public_key: String,
    
    /// User's verification method type
    #[clap(long, default_value = "Ed25519VerificationKey2020", help = "Verification method type")]
    pub user_key_type: String,
    
    /// User's verification method fragment
    #[clap(long, default_value = "user-key", help = "Verification method fragment")]
    pub user_fragment: String,
    
    /// Custodian's main DID string
    #[clap(long, help = "Custodian's main DID string")]
    pub custodian_did: String,
    
    /// Custodian's service public key
    #[clap(long, help = "Custodian's service public key in multibase format")]
    pub custodian_service_key: String,
    
    /// Custodian's service verification method type
    #[clap(long, default_value = "EcdsaSecp256k1VerificationKey2019", help = "Custodian service VM type")]
    pub custodian_key_type: String,
    
    /// Custodian's service verification method fragment
    #[clap(long, default_value = "service-key", help = "Custodian service VM fragment")]
    pub custodian_fragment: String,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOutput {
    pub did: String,
    pub object_id: String,
    pub transaction_hash: String,
    pub gas_used: u64,
    pub status: String,
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

        // Calculate the DID identifier
        let did_identifier = format!("did:rooch:{}", sender.to_bech32());

        // Extract the DID object ID from the changeset
        let output = result.output.ok_or_else(|| {
            rooch_types::error::RoochError::CommandArgumentError(
                "Transaction output not available".to_string(),
            )
        })?;
        let did_object_id = extract_new_did_object_id(&output.changeset, &sender.into())?;

        Ok(CreateOutput {
            did: did_identifier,
            object_id: format!("0x{}", did_object_id),
            transaction_hash: result.execution_info.tx_hash.to_string(),
            gas_used: result.execution_info.gas_used.into(),
            status: "success".to_string(),
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

        // Create the CADOP DID creation action
        let user_did_key_string = MoveString::from_str(&self.user_did_key)?;
        let user_vm_pk_multibase = MoveString::from_str(&self.user_public_key)?;
        let user_vm_type = MoveString::from_str(&self.user_key_type)?;
        let user_vm_fragment = MoveString::from_str(&self.user_fragment)?;
        let custodian_main_did_string = MoveString::from_str(&self.custodian_did)?;
        let custodian_service_pk_multibase = MoveString::from_str(&self.custodian_service_key)?;
        let custodian_service_vm_type = MoveString::from_str(&self.custodian_key_type)?;
        let custodian_service_vm_fragment = MoveString::from_str(&self.custodian_fragment)?;

        let action = DIDModule::create_did_object_via_cadop_action(
            user_did_key_string,
            user_vm_pk_multibase,
            user_vm_type,
            user_vm_fragment,
            custodian_main_did_string,
            custodian_service_pk_multibase,
            custodian_service_vm_type,
            custodian_service_vm_fragment,
        );

        // Execute the transaction
        let tx_data = context
            .build_tx_data(sender, action, max_gas_amount)
            .await?;
        let result = context.sign_and_execute(sender, tx_data).await?;
        context.assert_execute_success(result.clone())?;

        // For CADOP, extract the new DID account address from changeset
        let output = result.output.ok_or_else(|| {
            rooch_types::error::RoochError::CommandArgumentError(
                "Transaction output not available".to_string(),
            )
        })?;
        let new_did_account_address = extract_new_account_address_from_changeset(&output.changeset)?;
        let did_identifier = format!("did:rooch:{}", RoochAddress::from(new_did_account_address).to_bech32());
        
        // Extract the DID object ID from the changeset
        let did_object_id = extract_new_did_object_id(&output.changeset, &new_did_account_address)?;

        Ok(CreateOutput {
            did: did_identifier,
            object_id: format!("0x{}", did_object_id),
            transaction_hash: result.execution_info.tx_hash.to_string(),
            gas_used: result.execution_info.gas_used.into(),
            status: "success".to_string(),
        })
    }
}

/// Extract the newly created DID object ID from the changeset
fn extract_new_did_object_id(
    changeset: &StateChangeSetView,
    did_account_address: &move_core_types::account_address::AccountAddress,
) -> RoochResult<moveos_types::moveos_std::object::ObjectID> {
    use moveos_types::moveos_std::object::ObjectID;
    use rooch_types::address::RoochAddress;
    use moveos_types::state::MoveStructType;
    
    // Calculate the expected DID identifier string
    let did_identifier_str = RoochAddress::from(*did_account_address).to_bech32();
    
    // Calculate the expected DID object ID using the same method as in did.move
    let did_document_struct_tag = rooch_types::framework::did::DIDDocument::struct_tag();
    let expected_object_id = moveos_types::moveos_std::object::custom_object_id(&did_identifier_str, &did_document_struct_tag);
    
    // Look for this object ID in the changeset
    for object_change in &changeset.changes {
        if let Some(op) = &object_change.value {
            if matches!(op, OpView::New(_)) {
                // Parse the object ID from the metadata
                if let Ok(object_id) = ObjectID::from_str(&object_change.metadata.id.to_string()) {
                    if object_id == expected_object_id {
                        return Ok(expected_object_id);
                    }
                }
            }
        }
    }
    
    Err(rooch_types::error::RoochError::CommandArgumentError(
        "Failed to find newly created DID object in changeset".to_string(),
    ))
}

/// Extract the new account address created during CADOP from the changeset
fn extract_new_account_address_from_changeset(
    changeset: &StateChangeSetView,
) -> RoochResult<move_core_types::account_address::AccountAddress> {
    use moveos_types::moveos_std::object::ObjectID;
    
    // Look for newly created Account objects in the changeset
    for object_change in &changeset.changes {
        if let Some(op) = &object_change.value {
            if matches!(op, OpView::New(_)) {
                // Check if this is an Account object by examining the object type
                if object_change.metadata.object_type.to_string().contains("Account") {
                    // Parse the object ID and extract the account address
                    if let Ok(object_id) = ObjectID::from_str(&object_change.metadata.id.to_string()) {
                        // For account objects, we need to extract the address from the object ID
                        // Since we can't access the internal path directly, we'll use a different approach
                        // The object ID string representation contains the address information
                        let object_id_str = object_id.to_string();
                        // Remove the "0x" prefix and take the first 32 bytes (64 hex chars) as the address
                        if let Some(hex_str) = object_id_str.strip_prefix("0x") {
                            if hex_str.len() >= 64 {
                                let addr_hex = &hex_str[0..64];
                                if let Ok(account_address) = move_core_types::account_address::AccountAddress::from_hex_literal(&format!("0x{}", addr_hex)) {
                                    return Ok(account_address);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err(rooch_types::error::RoochError::CommandArgumentError(
        "Failed to find newly created account in changeset".to_string(),
    ))
} 