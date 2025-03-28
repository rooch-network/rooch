// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{BytesView, StrView};
use anyhow::Result;
use move_binary_format::file_format::Ability;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
    u256,
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::type_info::TypeInfo;
use moveos_types::transaction::MoveAction;
use moveos_types::{
    access_path::AccessPath,
    move_types::FunctionId,
    transaction::{FunctionCall, ScriptCall},
};
use moveos_types::{
    move_std::{ascii::MoveAsciiString, string::MoveString},
    state::MoveStructType,
};
use moveos_types::{move_types::parse_module_id, moveos_std::decimal_value::DecimalValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

use super::{decimal_value_view::DecimalValueView, move_option_view::MoveOptionView};

pub type ModuleIdView = StrView<ModuleId>;
pub type TypeTagView = StrView<TypeTag>;
pub type StructTagView = StrView<StructTag>;
pub type FunctionIdView = StrView<FunctionId>;
pub type AccessPathView = StrView<AccessPath>;
pub type IdentifierView = StrView<Identifier>;
pub type ObjectIDView = StrView<ObjectID>;

impl_str_view_for! {TypeTag StructTag FunctionId AccessPath Identifier ObjectID}

pub type AccountAddressView = StrView<AccountAddress>;

impl std::fmt::Display for AccountAddressView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //Ensure append `0x` before the address, and output full address
        //The Display implementation of AccountAddress has not `0x` prefix
        write!(f, "{:#x}", self.0)
    }
}

impl FromStr for AccountAddressView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // AccountAddress::from_str support both 0xADDRESS and ADDRESS
        Ok(StrView(AccountAddress::from_str(s)?))
    }
}

impl From<AccountAddressView> for AccountAddress {
    fn from(value: AccountAddressView) -> Self {
        value.0
    }
}

pub type ObjectIDVecView = StrView<Vec<ObjectID>>;

impl std::fmt::Display for ObjectIDVecView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //The ObjectID should display fully hex string with `0x` prefix
        let concated_str = self
            .0
            .iter()
            .map(|id| format!("{:?}", id))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{}", concated_str)
    }
}

impl FromStr for ObjectIDVecView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ids = s
            .split(',')
            .filter(|s| !s.is_empty())
            .map(ObjectID::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StrView(ids))
    }
}

impl From<ObjectIDVecView> for Vec<ObjectID> {
    fn from(value: ObjectIDVecView) -> Self {
        value.0
    }
}

pub type AbilityView = StrView<Ability>;

impl std::fmt::Display for AbilityView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Ability::Copy => write!(f, "copy"),
            Ability::Drop => write!(f, "drop"),
            Ability::Store => write!(f, "store"),
            Ability::Key => write!(f, "key"),
        }
    }
}

impl FromStr for AbilityView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "copy" => Ok(StrView(Ability::Copy)),
            "drop" => Ok(StrView(Ability::Drop)),
            "store" => Ok(StrView(Ability::Store)),
            "key" => Ok(StrView(Ability::Key)),
            _ => Err(anyhow::anyhow!("Invalid ability: {}", s)),
        }
    }
}

impl From<AbilityView> for Ability {
    fn from(value: AbilityView) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, Eq, PartialEq)]
pub struct AnnotatedMoveStructView {
    pub abilities: u8,
    #[serde(rename = "type")]
    pub type_: StructTagView,
    //We use BTreeMap to Replace Vec to make the output more readable
    pub value: BTreeMap<Identifier, AnnotatedMoveValueView>,
}

impl From<AnnotatedMoveStruct> for AnnotatedMoveStructView {
    fn from(origin: AnnotatedMoveStruct) -> Self {
        Self {
            abilities: origin.abilities.into_u8(),
            type_: StrView(origin.ty_tag),
            value: origin
                .value
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<AnnotatedMoveStructView> for serde_json::Value {
    fn from(value: AnnotatedMoveStructView) -> Self {
        let to_json_result = serde_json::to_value(value);
        debug_assert!(
            to_json_result.is_ok(),
            "AnnotatedMoveStructView to json failed"
        );
        to_json_result.unwrap_or(serde_json::Value::Null)
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, Eq, PartialEq)]
pub struct AnnotatedMoveStructVectorView {
    /// alilities of each element
    pub abilities: u8,
    #[serde(rename = "type")]
    /// type of each element
    pub type_: StructTagView,
    /// field of each element
    pub field: Vec<IdentifierView>,
    // values of the whole vector
    pub value: Vec<Vec<AnnotatedMoveValueView>>,
}

impl AnnotatedMoveStructVectorView {
    fn try_from(origin: Vec<AnnotatedMoveValue>) -> Result<Self, AnnotatedMoveValueView> {
        if origin.is_empty() {
            Err(AnnotatedMoveValueView::Vector(
                origin.into_iter().map(Into::into).collect(),
            ))
        } else {
            let first = origin.first().unwrap();
            if let AnnotatedMoveValue::Struct(ele) = first {
                //if the first element is a specific struct, we directly convert it to vector,
                //otherwise, we convert it to StructVector
                if SpecificStructView::try_from_annotated(ele).is_some() {
                    return Err(AnnotatedMoveValueView::Vector(
                        origin.into_iter().map(Into::into).collect(),
                    ));
                }
                let field = ele
                    .value
                    .iter()
                    .map(|x| IdentifierView::from(x.0.clone()))
                    .collect();
                let abilities = ele.abilities.into_u8();
                let type_ = StrView(ele.ty_tag.clone());
                let value: Vec<Vec<AnnotatedMoveValueView>> = origin
                    .into_iter()
                    .map(|v| {
                        if let AnnotatedMoveValue::Struct(s) = v {
                            s.value.into_iter().map(|(_, v)| v.into()).collect()
                        } else {
                            unreachable!("AnnotatedMoveStructVectorView")
                        }
                    })
                    .collect();

                Ok(Self {
                    abilities,
                    type_,
                    field,
                    value,
                })
            } else {
                Err(AnnotatedMoveValueView::Vector(
                    origin.into_iter().map(Into::into).collect(),
                ))
            }
        }
    }
}

/// Some specific struct that we want to display in a special way for better readability
#[derive(Debug, Clone, Serialize, JsonSchema, Eq, PartialEq)]
#[serde(untagged)]
pub enum SpecificStructView {
    MoveString(MoveString),
    MoveAsciiString(MoveAsciiString),
    ObjectID(ObjectID),
    DecimalValue(DecimalValueView),
    Option(MoveOptionView),
}

impl SpecificStructView {
    pub fn try_from_annotated(move_struct: &AnnotatedMoveStruct) -> Option<Self> {
        if MoveString::struct_tag_match(&move_struct.ty_tag) {
            MoveString::try_from(move_struct)
                .ok()
                .map(SpecificStructView::MoveString)
        } else if MoveAsciiString::struct_tag_match(&move_struct.ty_tag) {
            MoveAsciiString::try_from(move_struct)
                .ok()
                .map(SpecificStructView::MoveAsciiString)
        } else if ObjectID::struct_tag_match(&move_struct.ty_tag) {
            ObjectID::try_from(move_struct)
                .ok()
                .map(SpecificStructView::ObjectID)
        } else if DecimalValue::struct_tag_match(&move_struct.type_) {
            DecimalValue::try_from(move_struct)
                .ok()
                .map(DecimalValueView::from)
                .map(SpecificStructView::DecimalValue)
        } else if MoveOptionView::struct_tag_match(&move_struct.type_) {
            MoveOptionView::try_from(move_struct)
                .ok()
                .map(SpecificStructView::Option)
        } else {
            None
        }
    }
}

/// AnnotatedMoveValueView only used for serialization
#[derive(Debug, Clone, Serialize, JsonSchema, Eq, PartialEq)]
#[serde(untagged)]
pub enum AnnotatedMoveValueView {
    U8(u8),
    U16(u16),
    U32(u32),
    ///u64, u128, U256 is too large to be serialized in json
    /// so we use string to represent them
    U64(StrView<u64>),
    U128(StrView<u128>),
    U256(StrView<u256::U256>),
    Bool(bool),
    Address(AccountAddressView),
    Bytes(BytesView),
    SpecificStruct(Box<SpecificStructView>),
    Struct(AnnotatedMoveStructView),
    StructVector(Box<AnnotatedMoveStructVectorView>),
    Vector(Vec<AnnotatedMoveValueView>),
    // Add this variant as a "catch-all" for forward compatibility
    //Unknown(serde_json::Value),
}

impl AnnotatedMoveValueView {
    /// Calculate the total number of Move structs recursively
    pub fn size_of_struct_recursively(origin: &AnnotatedMoveValue) -> usize {
        match origin {
            AnnotatedMoveValue::Vector(_, data) => match data.first() {
                Some(first) => Self::size_of_struct_recursively(first) * data.len(),
                None => 0,
            },
            AnnotatedMoveValue::Struct(data) => {
                let mut size = 1;
                for (_, v) in &data.value {
                    size += Self::size_of_struct_recursively(v);
                }
                size
            }
            _ => 0,
        }
    }
}

impl From<AnnotatedMoveValue> for AnnotatedMoveValueView {
    fn from(origin: AnnotatedMoveValue) -> Self {
        match origin {
            AnnotatedMoveValue::U8(u) => AnnotatedMoveValueView::U8(u),
            AnnotatedMoveValue::U64(u) => AnnotatedMoveValueView::U64(StrView(u)),
            AnnotatedMoveValue::U128(u) => AnnotatedMoveValueView::U128(StrView(u)),
            AnnotatedMoveValue::Bool(b) => AnnotatedMoveValueView::Bool(b),
            AnnotatedMoveValue::Address(data) => AnnotatedMoveValueView::Address(StrView(data)),
            AnnotatedMoveValue::Vector(_type_tag, data) => {
                match AnnotatedMoveStructVectorView::try_from(data) {
                    Ok(v) => AnnotatedMoveValueView::StructVector(Box::new(v)),
                    Err(v) => v,
                }
            }
            AnnotatedMoveValue::Bytes(data) => AnnotatedMoveValueView::Bytes(StrView(data)),
            AnnotatedMoveValue::Struct(data) => {
                match SpecificStructView::try_from_annotated(&data) {
                    Some(struct_view) => {
                        AnnotatedMoveValueView::SpecificStruct(Box::new(struct_view))
                    }
                    None => AnnotatedMoveValueView::Struct(data.into()),
                }
            }
            AnnotatedMoveValue::U16(u) => AnnotatedMoveValueView::U16(u),
            AnnotatedMoveValue::U32(u) => AnnotatedMoveValueView::U32(u),
            AnnotatedMoveValue::U256(u) => AnnotatedMoveValueView::U256(StrView(u)),
        }
    }
}

impl From<AnnotatedMoveValueView> for serde_json::Value {
    fn from(value: AnnotatedMoveValueView) -> Self {
        let to_json_result = serde_json::to_value(value);
        debug_assert!(
            to_json_result.is_ok(),
            "AnnotatedMoveValueView to json failed"
        );
        to_json_result.unwrap_or(serde_json::Value::Null)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ScriptCallView {
    pub code: BytesView,
    pub ty_args: Vec<TypeTagView>,
    pub args: Vec<BytesView>,
}

impl From<ScriptCall> for ScriptCallView {
    fn from(origin: ScriptCall) -> Self {
        Self {
            code: origin.code.into(),
            ty_args: origin.ty_args.into_iter().map(StrView).collect(),
            args: origin.args.into_iter().map(StrView).collect(),
        }
    }
}

impl From<ScriptCallView> for ScriptCall {
    fn from(value: ScriptCallView) -> Self {
        Self {
            code: value.code.into(),
            ty_args: value.ty_args.into_iter().map(Into::into).collect(),
            args: value.args.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FunctionCallView {
    pub function_id: FunctionIdView,
    pub ty_args: Vec<TypeTagView>,
    pub args: Vec<BytesView>,
}

impl From<FunctionCall> for FunctionCallView {
    fn from(origin: FunctionCall) -> Self {
        Self {
            function_id: StrView(origin.function_id),
            ty_args: origin.ty_args.into_iter().map(StrView).collect(),
            args: origin.args.into_iter().map(StrView).collect(),
        }
    }
}

impl From<FunctionCallView> for FunctionCall {
    fn from(value: FunctionCallView) -> Self {
        Self {
            function_id: value.function_id.into(),
            ty_args: value.ty_args.into_iter().map(Into::into).collect(),
            args: value.args.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoveActionView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCallView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_call: Option<ScriptCallView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_bundle: Option<Vec<BytesView>>,
}

impl From<MoveAction> for MoveActionView {
    fn from(action: MoveAction) -> Self {
        match action {
            MoveAction::Script(script) => Self {
                script_call: Some(script.into()),
                function_call: None,
                module_bundle: None,
            },
            MoveAction::Function(fun) => Self {
                script_call: None,
                function_call: Some(fun.into()),
                module_bundle: None,
            },
            MoveAction::ModuleBundle(module) => Self {
                script_call: None,
                function_call: None,
                module_bundle: Some(module.into_iter().map(StrView).collect()),
            },
        }
    }
}

impl From<MoveActionView> for MoveAction {
    fn from(action: MoveActionView) -> Self {
        if let Some(script_call) = action.script_call {
            MoveAction::Script(script_call.into())
        } else if let Some(function_call) = action.function_call {
            MoveAction::Function(function_call.into())
        } else if let Some(module_bundle) = action.module_bundle {
            MoveAction::ModuleBundle(module_bundle.into_iter().map(StrView::into).collect())
        } else {
            panic!("Invalid MoveActionView")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MoveActionTypeView {
    ScriptCall,
    FunctionCall,
    ModuleBundle,
}

impl From<MoveAction> for MoveActionTypeView {
    fn from(action: MoveAction) -> Self {
        match action {
            MoveAction::Script(_) => Self::ScriptCall,
            MoveAction::Function(_) => Self::FunctionCall,
            MoveAction::ModuleBundle(_) => Self::ModuleBundle,
        }
    }
}

impl std::fmt::Display for StrView<ModuleId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.short_str_lossless())
    }
}

impl FromStr for StrView<ModuleId> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_module_id(s)?))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TypeInfoView {
    pub account_address: AccountAddress,
    pub module_name: BytesView,
    pub struct_name: BytesView,
}

impl std::fmt::Display for TypeInfoView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}::{:?}::{:?}",
            &self.account_address, self.module_name, &self.struct_name
        )
    }
}

impl From<TypeInfo> for TypeInfoView {
    fn from(type_info: TypeInfo) -> Self {
        TypeInfoView {
            account_address: type_info.account_address,
            module_name: type_info.module_name.into(),
            struct_name: type_info.struct_name.into(),
        }
    }
}

impl From<TypeInfoView> for TypeInfo {
    fn from(type_info: TypeInfoView) -> Self {
        TypeInfo {
            account_address: type_info.account_address,
            module_name: type_info.module_name.into(),
            struct_name: type_info.struct_name.into(),
        }
    }
}
