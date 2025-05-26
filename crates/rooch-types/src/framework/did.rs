// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::string::MoveString,
    moveos_std::object::ObjectID,
    moveos_std::tx_context::TxContext,
    state::{MoveState, MoveStructState, MoveStructType},
    transaction::{FunctionCall, MoveAction},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

pub const MODULE_NAME: &IdentStr = ident_str!("did");

/// DID identifier type
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct DID {
    pub method: MoveString,
    pub identifier: MoveString,
}

impl DID {
    pub fn new(method: &str, identifier: &str) -> Result<Self> {
        Ok(Self {
            method: MoveString::from_str(method)?,
            identifier: MoveString::from_str(identifier)?,
        })
    }

    pub fn parse(did_string: &str) -> Result<Self> {
        let parts: Vec<&str> = did_string.split(':').collect();
        if parts.len() < 3 || parts[0] != "did" {
            return Err(anyhow::anyhow!("Invalid DID format: {}", did_string));
        }
        
        let method = parts[1];
        let identifier = parts[2..].join(":");
        
        Self::new(method, &identifier)
    }

}

impl Display for DID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "did:{}:{}", self.method, self.identifier)
    }
}

impl FromStr for DID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl MoveStructType for DID {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DID");
}

impl MoveStructState for DID {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveString::type_layout(),
            MoveString::type_layout(),
        ])
    }
}

/// Verification method ID
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct VerificationMethodID {
    pub did: DID,
    pub fragment: MoveString,
}

impl VerificationMethodID {
    pub fn new(did: DID, fragment: &str) -> Result<Self> {
        Ok(Self {
            did,
            fragment: MoveString::from_str(fragment)?,
        })
    }

}

impl Display for VerificationMethodID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.did, self.fragment)
    }
}

impl MoveStructType for VerificationMethodID {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("VerificationMethodID");
}

impl MoveStructState for VerificationMethodID {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            DID::type_layout(),
            MoveString::type_layout(),
        ])
    }
}

/// Service ID
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ServiceID {
    pub did: DID,
    pub fragment: MoveString,
}

impl ServiceID {
    pub fn new(did: DID, fragment: &str) -> Result<Self> {
        Ok(Self {
            did,
            fragment: MoveString::from_str(fragment)?,
        })
    }

}

impl Display for ServiceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.did, self.fragment)
    }
}

impl MoveStructType for ServiceID {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("ServiceID");
}

impl MoveStructState for ServiceID {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            DID::type_layout(),
            MoveString::type_layout(),
        ])
    }
}

/// Verification method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: VerificationMethodID,
    pub method_type: MoveString,
    pub controller: DID,
    pub public_key_multibase: MoveString,
}

impl VerificationMethod {
    pub fn new(
        id: VerificationMethodID,
        method_type: &str,
        controller: DID,
        public_key_multibase: &str,
    ) -> Result<Self> {
        Ok(Self {
            id,
            method_type: MoveString::from_str(method_type)?,
            controller,
            public_key_multibase: MoveString::from_str(public_key_multibase)?,
        })
    }
}

impl MoveStructType for VerificationMethod {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("VerificationMethod");
}

impl MoveStructState for VerificationMethod {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            VerificationMethodID::type_layout(),
            MoveString::type_layout(),
            DID::type_layout(),
            MoveString::type_layout(),
        ])
    }
}

/// Service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: ServiceID,
    pub service_type: MoveString,
    pub service_endpoint: MoveString,
    pub properties: HashMap<String, String>,
}

impl Service {
    pub fn new(
        id: ServiceID,
        service_type: &str,
        service_endpoint: &str,
        properties: HashMap<String, String>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            service_type: MoveString::from_str(service_type)?,
            service_endpoint: MoveString::from_str(service_endpoint)?,
            properties,
        })
    }
}

impl MoveStructType for Service {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Service");
}

// Note: Service struct layout is complex due to properties HashMap, 
// implementing minimal version for now
impl MoveStructState for Service {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            ServiceID::type_layout(),
            MoveString::type_layout(),
            MoveString::type_layout(),
            // SimpleMap layout would be more complex, simplified for now
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8
            )),
        ])
    }
}

/// DID Document
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    pub id: DID,
    pub controller: Vec<DID>,
    pub verification_methods: HashMap<String, VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub capability_invocation: Vec<String>,
    pub capability_delegation: Vec<String>,
    pub key_agreement: Vec<String>,
    pub services: HashMap<String, Service>,
    pub also_known_as: Vec<String>,
}

impl DIDDocument {
    pub fn get_verification_method(&self, fragment: &str) -> Option<&VerificationMethod> {
        self.verification_methods.get(fragment)
    }

    pub fn get_service(&self, fragment: &str) -> Option<&Service> {
        self.services.get(fragment)
    }

    pub fn has_verification_relationship(&self, fragment: &str, relationship: VerificationRelationship) -> bool {
        let relationship_vec = match relationship {
            VerificationRelationship::Authentication => &self.authentication,
            VerificationRelationship::AssertionMethod => &self.assertion_method,
            VerificationRelationship::CapabilityInvocation => &self.capability_invocation,
            VerificationRelationship::CapabilityDelegation => &self.capability_delegation,
            VerificationRelationship::KeyAgreement => &self.key_agreement,
        };
        relationship_vec.contains(&fragment.to_string())
    }
}

impl MoveStructType for DIDDocument {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DIDDocument");
}

// DIDDocument has complex layout, simplified for basic functionality
impl MoveStructState for DIDDocument {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            DID::type_layout(),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(DID::type_layout())),
            // Simplified layouts for complex nested structures
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(MoveString::type_layout())),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(MoveString::type_layout())),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(MoveString::type_layout())),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(MoveString::type_layout())),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(MoveString::type_layout())),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(MoveString::type_layout())),
        ])
    }
}

/// Verification relationship types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum VerificationRelationship {
    Authentication = 0,
    AssertionMethod = 1,
    CapabilityInvocation = 2,
    CapabilityDelegation = 3,
    KeyAgreement = 4,
}

impl VerificationRelationship {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(VerificationRelationship::Authentication),
            1 => Ok(VerificationRelationship::AssertionMethod),
            2 => Ok(VerificationRelationship::CapabilityInvocation),
            3 => Ok(VerificationRelationship::CapabilityDelegation),
            4 => Ok(VerificationRelationship::KeyAgreement),
            _ => Err(anyhow::anyhow!("Invalid verification relationship: {}", value)),
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            VerificationRelationship::Authentication => "authentication",
            VerificationRelationship::AssertionMethod => "assertionMethod",
            VerificationRelationship::CapabilityInvocation => "capabilityInvocation",
            VerificationRelationship::CapabilityDelegation => "capabilityDelegation",
            VerificationRelationship::KeyAgreement => "keyAgreement",
        }
    }

    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "authentication" | "auth" => Ok(VerificationRelationship::Authentication),
            "assertionmethod" | "assert" => Ok(VerificationRelationship::AssertionMethod),
            "capabilityinvocation" | "invoke" => Ok(VerificationRelationship::CapabilityInvocation),
            "capabilitydelegation" | "delegate" => Ok(VerificationRelationship::CapabilityDelegation),
            "keyagreement" | "agreement" => Ok(VerificationRelationship::KeyAgreement),
            _ => Err(anyhow::anyhow!("Invalid verification relationship: {}", s)),
        }
    }
}

impl Display for VerificationRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FromStr for VerificationRelationship {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

/// Verification method types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum VerificationMethodType {
    Ed25519VerificationKey2020,
    EcdsaSecp256k1VerificationKey2019,
}

impl VerificationMethodType {
    pub fn to_string(&self) -> &'static str {
        match self {
            VerificationMethodType::Ed25519VerificationKey2020 => "Ed25519VerificationKey2020",
            VerificationMethodType::EcdsaSecp256k1VerificationKey2019 => "EcdsaSecp256k1VerificationKey2019",
        }
    }

    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "Ed25519VerificationKey2020" | "ed25519" => Ok(VerificationMethodType::Ed25519VerificationKey2020),
            "EcdsaSecp256k1VerificationKey2019" | "secp256k1" => Ok(VerificationMethodType::EcdsaSecp256k1VerificationKey2019),
            _ => Err(anyhow::anyhow!("Invalid verification method type: {}", s)),
        }
    }
}

impl Display for VerificationMethodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FromStr for VerificationMethodType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

/// Rust bindings for RoochFramework did module
pub struct DIDModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> DIDModule<'a> {
    // Entry function names from the Move module
    pub const CREATE_DID_OBJECT_FOR_SELF_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("create_did_object_for_self_entry");
    pub const CREATE_DID_OBJECT_VIA_CADOP_WITH_DID_KEY_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("create_did_object_via_cadop_with_did_key_entry");
    pub const ADD_VERIFICATION_METHOD_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("add_verification_method_entry");
    pub const REMOVE_VERIFICATION_METHOD_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("remove_verification_method_entry");
    pub const ADD_TO_VERIFICATION_RELATIONSHIP_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("add_to_verification_relationship_entry");
    pub const REMOVE_FROM_VERIFICATION_RELATIONSHIP_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("remove_from_verification_relationship_entry");
    pub const ADD_SERVICE_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("add_service_entry");
    pub const ADD_SERVICE_WITH_PROPERTIES_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("add_service_with_properties_entry");
    pub const UPDATE_SERVICE_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("update_service_entry");
    pub const REMOVE_SERVICE_ENTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("remove_service_entry");
    pub const INIT_DID_REGISTRY_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("init_did_registry");

    // Query function names
    pub const EXISTS_DID_DOCUMENT_BY_IDENTIFIER_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("exists_did_document_by_identifier");
    pub const EXISTS_DID_FOR_ADDRESS_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("exists_did_for_address");
    pub const GET_DIDS_BY_CONTROLLER_STRING_FUNCTION_NAME: &'static IdentStr = 
        ident_str!("get_dids_by_controller_string");

    /// Create DID action for self
    pub fn create_did_object_for_self_action(
        account_public_key_multibase: MoveString,
    ) -> MoveAction {
        Self::create_move_action(
            Self::CREATE_DID_OBJECT_FOR_SELF_ENTRY_FUNCTION_NAME,
            vec![],
            vec![account_public_key_multibase.to_move_value()],
        )
    }

    /// Create DID action via CADOP with did:key
    pub fn create_did_object_via_cadop_with_did_key_action(
        user_did_key_string: MoveString,
        custodian_service_pk_multibase: MoveString,
        custodian_service_vm_type: MoveString,
    ) -> MoveAction {
        Self::create_move_action(
            Self::CREATE_DID_OBJECT_VIA_CADOP_WITH_DID_KEY_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                user_did_key_string.to_move_value(),
                custodian_service_pk_multibase.to_move_value(),
                custodian_service_vm_type.to_move_value(),
            ],
        )
    }

    /// Add verification method action
    pub fn add_verification_method_action(
        fragment: MoveString,
        method_type: MoveString,
        public_key_multibase: MoveString,
        verification_relationships: Vec<u8>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::ADD_VERIFICATION_METHOD_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                fragment.to_move_value(),
                method_type.to_move_value(),
                public_key_multibase.to_move_value(),
                move_core_types::value::MoveValue::Vector(
                    verification_relationships.into_iter()
                        .map(move_core_types::value::MoveValue::U8)
                        .collect()
                ),
            ],
        )
    }

    /// Remove verification method action
    pub fn remove_verification_method_action(fragment: MoveString) -> MoveAction {
        Self::create_move_action(
            Self::REMOVE_VERIFICATION_METHOD_ENTRY_FUNCTION_NAME,
            vec![],
            vec![fragment.to_move_value()],
        )
    }

    /// Add to verification relationship action
    pub fn add_to_verification_relationship_action(
        fragment: MoveString,
        relationship_type: u8,
    ) -> MoveAction {
        Self::create_move_action(
            Self::ADD_TO_VERIFICATION_RELATIONSHIP_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                fragment.to_move_value(),
                move_core_types::value::MoveValue::U8(relationship_type),
            ],
        )
    }

    /// Remove from verification relationship action
    pub fn remove_from_verification_relationship_action(
        fragment: MoveString,
        relationship_type: u8,
    ) -> MoveAction {
        Self::create_move_action(
            Self::REMOVE_FROM_VERIFICATION_RELATIONSHIP_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                fragment.to_move_value(),
                move_core_types::value::MoveValue::U8(relationship_type),
            ],
        )
    }

    /// Add service action
    pub fn add_service_action(
        fragment: MoveString,
        service_type: MoveString,
        service_endpoint: MoveString,
    ) -> MoveAction {
        Self::create_move_action(
            Self::ADD_SERVICE_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                fragment.to_move_value(),
                service_type.to_move_value(),
                service_endpoint.to_move_value(),
            ],
        )
    }

    /// Add service with properties action
    pub fn add_service_with_properties_action(
        fragment: MoveString,
        service_type: MoveString,
        service_endpoint: MoveString,
        property_keys: Vec<MoveString>,
        property_values: Vec<MoveString>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::ADD_SERVICE_WITH_PROPERTIES_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                fragment.to_move_value(),
                service_type.to_move_value(),
                service_endpoint.to_move_value(),
                move_core_types::value::MoveValue::Vector(
                    property_keys.into_iter()
                        .map(|k| k.to_move_value())
                        .collect()
                ),
                move_core_types::value::MoveValue::Vector(
                    property_values.into_iter()
                        .map(|v| v.to_move_value())
                        .collect()
                ),
            ],
        )
    }

    /// Remove service action
    pub fn remove_service_action(fragment: MoveString) -> MoveAction {
        Self::create_move_action(
            Self::REMOVE_SERVICE_ENTRY_FUNCTION_NAME,
            vec![],
            vec![fragment.to_move_value()],
        )
    }

    /// Initialize DID registry action
    pub fn init_did_registry_action() -> MoveAction {
        Self::create_move_action(
            Self::INIT_DID_REGISTRY_FUNCTION_NAME,
            vec![],
            vec![],
        )
    }

    /// Check if DID document exists by identifier
    pub fn exists_did_document_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<bool> {
        let call = FunctionCall::new(
            Self::function_id(Self::EXISTS_DID_DOCUMENT_BY_IDENTIFIER_FUNCTION_NAME),
            vec![],
            vec![MoveString::from_str(identifier)?.to_move_value().simple_serialize().unwrap()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let exists = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<bool>(&value.value)
                    .expect("should be a valid bool")
            })?;
        Ok(exists)
    }

    /// Check if DID exists for address
    pub fn exists_did_for_address(
        &self,
        address: AccountAddress,
    ) -> Result<bool> {
        let call = FunctionCall::new(
            Self::function_id(Self::EXISTS_DID_FOR_ADDRESS_FUNCTION_NAME),
            vec![],
            vec![move_core_types::value::MoveValue::Address(address).simple_serialize().unwrap()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let exists = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<bool>(&value.value)
                    .expect("should be a valid bool")
            })?;
        Ok(exists)
    }

    /// Get DIDs controlled by a specific controller DID
    pub fn get_dids_by_controller_string(
        &self,
        controller_did_str: &str,
    ) -> Result<Vec<ObjectID>> {
        let call = FunctionCall::new(
            Self::function_id(Self::GET_DIDS_BY_CONTROLLER_STRING_FUNCTION_NAME),
            vec![],
            vec![MoveString::from_str(controller_did_str)?.to_move_value().simple_serialize().unwrap()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let object_ids = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<Vec<ObjectID>>(&value.value)
                    .expect("should be a valid Vec<ObjectID>")
            })?;
        Ok(object_ids)
    }
}

impl<'a> ModuleBinding<'a> for DIDModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_parsing() {
        let did_str = "did:rooch:bc1q2dmjktatkwyf";
        let did = DID::parse(did_str).unwrap();
        assert_eq!(did.method.as_str(), "rooch");
        assert_eq!(did.identifier.as_str(), "bc1q2dmjktatkwyf");
        assert_eq!(did.to_string(), did_str);
    }

    #[test]
    fn test_did_key_parsing() {
        let did_str = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
        let did = DID::parse(did_str).unwrap();
        assert_eq!(did.method.as_str(), "key");
        assert_eq!(did.identifier.as_str(), "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH");
    }

    #[test]
    fn test_verification_relationship_parsing() {
        assert_eq!(
            VerificationRelationship::from_string("auth").unwrap(),
            VerificationRelationship::Authentication
        );
        assert_eq!(
            VerificationRelationship::from_string("delegate").unwrap(),
            VerificationRelationship::CapabilityDelegation
        );
        assert_eq!(
            VerificationRelationship::from_string("invoke").unwrap(),
            VerificationRelationship::CapabilityInvocation
        );
    }

    #[test]
    fn test_verification_method_type_parsing() {
        assert_eq!(
            VerificationMethodType::from_string("ed25519").unwrap(),
            VerificationMethodType::Ed25519VerificationKey2020
        );
        assert_eq!(
            VerificationMethodType::from_string("secp256k1").unwrap(),
            VerificationMethodType::EcdsaSecp256k1VerificationKey2019
        );
    }

    #[test]
    fn test_verification_method_id_display() {
        let did = DID::new("rooch", "bc1q2dmjktatkwyf").unwrap();
        let vm_id = VerificationMethodID::new(did, "key-1").unwrap();
        assert_eq!(vm_id.to_string(), "did:rooch:bc1q2dmjktatkwyf#key-1");
    }

    #[test]
    fn test_invalid_did_format() {
        assert!(DID::parse("invalid").is_err());
        assert!(DID::parse("did:rooch").is_err());
        assert!(DID::parse("not:did:format").is_err());
    }
}
