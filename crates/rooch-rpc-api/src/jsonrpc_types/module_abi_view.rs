// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AbilityView, AccountAddressView, IdentifierView, ModuleIdView, TypeTagView,
};
use anyhow::Result;
use move_binary_format::{
    access::ModuleAccess,
    file_format::{
        AbilitySet, CompiledModule, FieldDefinition, FunctionDefinition, SignatureToken,
        StructDefinition, StructFieldInformation, StructHandleIndex, StructTypeParameter,
        Visibility,
    },
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::Debug;

fn signature_token_to_struct_tag(
    m: &CompiledModule,
    index: &StructHandleIndex,
    type_params: &[SignatureToken],
) -> StructTag {
    let s_handle = m.struct_handle_at(*index);
    let m_handle = m.module_handle_at(s_handle.module);
    StructTag {
        address: (*m.address_identifier_at(m_handle.address)).into(),
        module: m.identifier_at(m_handle.name).to_owned().into(),
        name: m.identifier_at(s_handle.name).to_owned().into(),
        type_params: type_params
            .iter()
            .map(|t| signature_token_to_type_tag(m, t))
            .collect(),
    }
}

fn signature_token_to_type_tag(m: &CompiledModule, token: &SignatureToken) -> TypeTag {
    match token {
        SignatureToken::Bool => TypeTag::Bool,
        SignatureToken::U8 => TypeTag::U8,
        SignatureToken::U16 => TypeTag::U16,
        SignatureToken::U32 => TypeTag::U32,
        SignatureToken::U64 => TypeTag::U64,
        SignatureToken::U128 => TypeTag::U128,
        SignatureToken::U256 => TypeTag::U256,
        SignatureToken::Address => TypeTag::Address,
        SignatureToken::Signer => TypeTag::Signer,
        SignatureToken::Vector(t) => {
            TypeTag::Vector(Box::new(signature_token_to_type_tag(m, t.borrow())))
        }
        SignatureToken::Struct(v) => TypeTag::from(signature_token_to_struct_tag(&m, v, &[])),
        SignatureToken::StructInstantiation(shi, type_params) => {
            TypeTag::from(signature_token_to_struct_tag(m, shi, type_params))
        }
        // SignatureToken::TypeParameter(i) => MoveType::GenericTypeParam { index: *i },
        // SignatureToken::Reference(t) => MoveType::Reference {
        //     mutable: false,
        //     to: Box::new(self.new_move_type(t.borrow())),
        // },
        // SignatureToken::MutableReference(t) => MoveType::Reference {
        //     mutable: true,
        //     to: Box::new(self.new_move_type(t.borrow())),
        // },
        _ => todo!(),
    }
}

/// Move function generic type param
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MoveFunctionTypeParamView {
    /// Move abilities tied to the generic type param and associated with the function that uses it
    pub constraints: Vec<AbilityView>,
}

impl From<&AbilitySet> for MoveFunctionTypeParamView {
    fn from(constraints: &AbilitySet) -> Self {
        Self {
            constraints: constraints.into_iter().map(AbilityView::from).collect(),
        }
    }
}

/// Move function
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MoveFunctionView {
    pub name: IdentifierView,
    /// Whether the function can be called as an entry function directly in a transaction
    pub is_entry: bool,
    /// Generic type params associated with the Move function
    pub type_params: Vec<MoveFunctionTypeParamView>,
    /// Parameters associated with the move function
    pub params: Vec<TypeTagView>,
    /// Return type of the function
    pub return_: Vec<TypeTagView>,
}

impl MoveFunctionView {
    fn new(m: &CompiledModule, def: &FunctionDefinition) -> Self {
        let fhandle = m.function_handle_at(def.function);
        let name = m.identifier_at(fhandle.name).to_owned();
        Self {
            name: name.into(),
            is_entry: def.is_entry,
            type_params: fhandle
                .type_parameters
                .iter()
                .map(MoveFunctionTypeParamView::from)
                .collect(),
            params: m
                .signature_at(fhandle.parameters)
                .0
                .iter()
                .map(|s| TypeTagView::from(signature_token_to_type_tag(m, s)))
                .collect(),
            return_: m
                .signature_at(fhandle.return_)
                .0
                .iter()
                .map(|s| TypeTagView::from(signature_token_to_type_tag(m, s)))
                .collect(),
        }
    }
}

/// Move generic type param
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MoveStructTypeParamView {
    /// Move abilities tied to the generic type param and associated with the type that uses it
    pub constraints: Vec<AbilityView>,
    /// Whether the type is a phantom type
    pub is_phantom: bool,
}

impl From<&StructTypeParameter> for MoveStructTypeParamView {
    fn from(param: &StructTypeParameter) -> Self {
        Self {
            constraints: param
                .constraints
                .into_iter()
                .map(AbilityView::from)
                .collect(),
            is_phantom: param.is_phantom,
        }
    }
}

/// Move struct field
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MoveStructFieldView {
    pub name: IdentifierView,
    #[serde(rename = "type")]
    pub ty: TypeTagView,
}

fn new_move_struct_field_view(m: &CompiledModule, def: &FieldDefinition) -> MoveStructFieldView {
    MoveStructFieldView {
        name: m.identifier_at(def.name).to_owned().into(),
        ty: signature_token_to_type_tag(m, &def.signature.0).into(),
    }
}

/// A move struct
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MoveStructView {
    pub name: IdentifierView,
    /// Whether the struct is a native struct of Move
    pub is_native: bool,
    /// Abilities associated with the struct
    pub abilities: Vec<AbilityView>,
    /// Generic types associated with the struct
    pub type_params: Vec<MoveStructTypeParamView>,
    /// Fields associated with the struct
    pub fields: Vec<MoveStructFieldView>,
}

impl MoveStructView {
    fn new(m: &CompiledModule, def: &StructDefinition) -> Self {
        let handle = m.struct_handle_at(def.struct_handle);
        let name = m.identifier_at(handle.name).to_owned();

        let (is_native, fields) = match &def.field_information {
            StructFieldInformation::Native => (true, vec![]),
            StructFieldInformation::Declared(fields) => (
                false,
                fields
                    .iter()
                    .map(|f| new_move_struct_field_view(m, f))
                    .collect(),
            ),
        };

        let abilities = handle
            .abilities
            .into_iter()
            .map(AbilityView::from)
            .collect();
        let type_params = handle
            .type_parameters
            .iter()
            .map(MoveStructTypeParamView::from)
            .collect();
        Self {
            name: name.into(),
            is_native,
            abilities,
            type_params,
            fields,
        }
    }
}

/// A Move module ABI
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ModuleABIView {
    pub address: AccountAddressView,
    pub name: IdentifierView,
    /// Friends of the module
    pub friends: Vec<ModuleIdView>,
    /// Public or entry functions of the module
    pub functions: Vec<MoveFunctionView>,
    /// Structs of the module
    pub structs: Vec<MoveStructView>,
}

impl ModuleABIView {
    pub fn try_parse_from_module_bytes(module_bytes: &[u8]) -> Result<Self> {
        Ok(CompiledModule::deserialize(module_bytes)?.into())
    }
}

impl From<CompiledModule> for ModuleABIView {
    fn from(m: CompiledModule) -> Self {
        let (address, name) = <(AccountAddress, Identifier)>::from(m.self_id());
        Self {
            address: address.into(),
            name: name.into(),
            friends: m
                .immediate_friends()
                .into_iter()
                .map(|m| ModuleIdView::from(m))
                .collect(),
            functions: m
                .function_defs
                .iter()
                // Return all entry or public functions.
                // Private entry functions are still callable by entry function transactions so
                // they should be included.
                // friend functions are treated as private functions.
                // TODO: should friend functions be included?
                .filter(|def| {
                    def.is_entry
                        || match def.visibility {
                            Visibility::Public => true,
                            Visibility::Private | Visibility::Friend => false,
                        }
                })
                .map(|def| MoveFunctionView::new(&m, def))
                .collect(),
            structs: m
                .struct_defs
                .iter()
                .map(|def| MoveStructView::new(&m, def))
                .collect(),
        }
    }
}

// TODO: do we need to support export ABI of CompiledScript.
