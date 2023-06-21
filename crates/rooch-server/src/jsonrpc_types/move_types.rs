// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::StrView;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
    u256,
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use moveos_types::event::{Event, EventID};
use moveos_types::move_types::parse_module_id;
use moveos_types::transaction::MoveAction;
use moveos_types::{
    access_path::AccessPath,
    event_filter::EventFilter,
    h256::H256,
    move_types::FunctionId,
    object::{AnnotatedObject, ObjectID},
    serde::Readable,
    transaction::{FunctionCall, ScriptCall},
};

use fastcrypto::encoding::Hex;
use serde_with::serde_as;

use moveos_types::{
    move_string::{MoveAsciiString, MoveString},
    state::MoveStructState,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::str::FromStr;

pub type ModuleIdView = StrView<ModuleId>;
pub type TypeTagView = StrView<TypeTag>;
pub type StructTagView = StrView<StructTag>;
pub type FunctionIdView = StrView<FunctionId>;
pub type AccessPathView = StrView<AccessPath>;
pub type AccountAddressView = StrView<AccountAddress>;

impl_str_view_for! {TypeTag StructTag FunctionId AccessPath}
// impl_str_view_for! {TypeTag StructTag ModuleId FunctionId}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
            type_: StrView(origin.type_),
            value: origin
                .value
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

//TODO
// impl TryFrom<AnnotatedMoveStructView> for AnnotatedMoveStruct {
//     type Error = anyhow::Error;
//
//     fn try_from(value: AnnotatedMoveStructView) -> Result<Self, Self::Error> {
//         Ok(Self {
//             abilities: AbilitySet::from_u8(value.abilities)
//                 .ok_or_else(|| anyhow::anyhow!("invalid abilities:{}", value.abilities))?,
//             type_: value.type_.0,
//             value: value
//                 .value
//                 .into_iter()
//                 .map(|(k, v)| {
//                     Ok::<(Identifier, AnnotatedMoveValue), anyhow::Error>((k, v.try_into()?))
//                 })
//                 .collect::<Result<_, _>>()?,
//         })
//     }
// }

/// Some specific struct that we want to display in a special way for better readability
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum SpecificStructView {
    MoveString(MoveString),
    MoveAsciiString(MoveAsciiString),
    ObjectID(ObjectID),
}

impl SpecificStructView {
    pub fn try_from_annotated(move_struct: AnnotatedMoveStruct) -> Option<Self> {
        if MoveString::type_match(&move_struct.type_) {
            MoveString::try_from(move_struct)
                .ok()
                .map(SpecificStructView::MoveString)
        } else if MoveAsciiString::type_match(&move_struct.type_) {
            MoveAsciiString::try_from(move_struct)
                .ok()
                .map(SpecificStructView::MoveAsciiString)
        } else if ObjectID::type_match(&move_struct.type_) {
            ObjectID::try_from(move_struct)
                .ok()
                .map(SpecificStructView::ObjectID)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AnnotatedMoveValueView {
    U8(u8),
    ///u64, u128, U256 is too large to be serialized in json
    /// so we use string to represent them
    U64(StrView<u64>),
    U128(StrView<u128>),
    Bool(bool),
    Address(AccountAddressView),
    Vector(Vec<AnnotatedMoveValueView>),
    Bytes(StrView<Vec<u8>>),
    Struct(AnnotatedMoveStructView),
    SpecificStruct(SpecificStructView),
    U16(u16),
    U32(u32),
    U256(StrView<u256::U256>),
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
                AnnotatedMoveValueView::Vector(data.into_iter().map(Into::into).collect())
            }
            AnnotatedMoveValue::Bytes(data) => AnnotatedMoveValueView::Bytes(StrView(data)),
            AnnotatedMoveValue::Struct(data) => {
                match SpecificStructView::try_from_annotated(data.clone()) {
                    Some(struct_view) => AnnotatedMoveValueView::SpecificStruct(struct_view),
                    None => AnnotatedMoveValueView::Struct(data.into()),
                }
            }
            AnnotatedMoveValue::U16(u) => AnnotatedMoveValueView::U16(u),
            AnnotatedMoveValue::U32(u) => AnnotatedMoveValueView::U32(u),
            AnnotatedMoveValue::U256(u) => AnnotatedMoveValueView::U256(StrView(u)),
        }
    }
}

//TODO
// impl TryFrom<AnnotatedMoveValueView> for AnnotatedMoveValue {
//     type Error = anyhow::Error;
//     fn try_from(value: AnnotatedMoveValueView) -> Result<Self, Self::Error> {
//         Ok(match value {
//             AnnotatedMoveValueView::U8(u8) => AnnotatedMoveValue::U8(u8),
//             AnnotatedMoveValueView::U64(u64) => AnnotatedMoveValue::U64(u64.0),
//             AnnotatedMoveValueView::U128(u128) => AnnotatedMoveValue::U128(u128.0),
//             AnnotatedMoveValueView::Bool(bool) => AnnotatedMoveValue::Bool(bool),
//             AnnotatedMoveValueView::Address(address) => AnnotatedMoveValue::Address(address),
//             AnnotatedMoveValueView::Vector(type_tag, data) => AnnotatedMoveValue::Vector(
//                 type_tag.0,
//                 data.into_iter()
//                     .map(AnnotatedMoveValue::try_from)
//                     .collect::<Result<Vec<_>, Self::Error>>()?,
//             ),
//             AnnotatedMoveValueView::Bytes(data) => AnnotatedMoveValue::Bytes(data.0),
//             AnnotatedMoveValueView::Struct(data) => AnnotatedMoveValue::Struct(data.try_into()?),
//             AnnotatedMoveValueView::U16(u16) => AnnotatedMoveValue::U16(u16),
//             AnnotatedMoveValueView::U32(u32) => AnnotatedMoveValue::U32(u32),
//             AnnotatedMoveValueView::U256(u256) => AnnotatedMoveValue::U256(u256.0),
//         })
//     }
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnnotatedObjectView {
    pub id: ObjectID,
    pub owner: AccountAddressView,
    pub value: AnnotatedMoveStructView,
}

impl From<AnnotatedObject> for AnnotatedObjectView {
    fn from(origin: AnnotatedObject) -> Self {
        Self {
            id: origin.id,
            owner: origin.owner.into(),
            value: origin.value.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ScriptCallView {
    pub code: StrView<Vec<u8>>,
    pub ty_args: Vec<TypeTagView>,
    pub args: Vec<StrView<Vec<u8>>>,
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
    pub args: Vec<StrView<Vec<u8>>>,
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
    pub module_bundle: Option<Vec<StrView<Vec<u8>>>>,
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
        write!(f, "{}", &self.0)
    }
}

impl FromStr for StrView<ModuleId> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_module_id(s)?))
    }
}

// #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
// #[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
// pub struct Identifier(Box<str>);

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IdentifierView(String);

impl From<Identifier> for IdentifierView {
    fn from(value: Identifier) -> Self {
        IdentifierView(value.into_string())
    }
}

impl From<IdentifierView> for Identifier {
    fn from(value: IdentifierView) -> Self {
        Identifier::new(value.0).unwrap()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct H256View(
    #[schemars(with = "Hex")]
    #[serde_as(as = "Readable<Hex, _>")]
    [u8; 32],
);

impl From<H256> for H256View {
    fn from(value: H256) -> Self {
        H256View(value.0)
    }
}

impl From<H256View> for H256 {
    fn from(value: H256View) -> Self {
        H256(value.0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub enum EventFilterView {
    /// Query by sender address.
    Sender(AccountAddressView),
    /// Return events emitted by the given transaction.
    Transaction(
        ///tx hash of the transaction
        H256View,
    ),
    /// Return events with the given move event struct name
    MoveEventType(
        // #[schemars(with = "String")]
        // #[serde_as(as = "TypeTag")]
        TypeTagView,
    ),
    MoveEventField {
        path: String,
        value: Value,
    },
    /// Return events emitted in [start_time, end_time) interval
    // #[serde(rename_all = "camelCase")]
    TimeRange {
        /// left endpoint of time interval, milliseconds since epoch, inclusive
        // #[schemars(with = "u64")]
        // #[serde_as(as = "u64")]
        start_time: u64,
        /// right endpoint of time interval, milliseconds since epoch, exclusive
        // #[schemars(with = "u64")]
        // #[serde_as(as = "u64")]
        end_time: u64,
    },
    /// Return events emitted in [from_block, to_block) interval
    // #[serde(rename_all = "camelCase")]
    // BlockRange {
    //     /// left endpoint of block height, inclusive
    //     // #[schemars(with = "u64")]
    //     // #[serde_as(as = "u64")]
    //     from_block: u64, //TODO use BlockNumber
    //     /// right endpoint of block height, exclusive
    //     // #[schemars(with = "u64")]
    //     // #[serde_as(as = "u64")]
    //     to_block: u64, //TODO use BlockNumber
    // },
    All(Vec<EventFilterView>),
    Any(Vec<EventFilterView>),
    And(Box<EventFilterView>, Box<EventFilterView>),
    Or(Box<EventFilterView>, Box<EventFilterView>),
}

impl From<EventFilterView> for EventFilter {
    fn from(value: EventFilterView) -> Self {
        match value {
            EventFilterView::Sender(address) => Self::Sender(address.into()),
            EventFilterView::Transaction(tx_hash) => Self::Transaction(tx_hash.into()),
            EventFilterView::MoveEventType(type_tag) => Self::MoveEventType(type_tag.into()),
            EventFilterView::MoveEventField { path, value } => Self::MoveEventField { path, value },
            EventFilterView::TimeRange {
                start_time,
                end_time,
            } => Self::TimeRange {
                start_time,
                end_time,
            },
            EventFilterView::All(filters) => {
                Self::All(filters.into_iter().map(|f| f.into()).collect())
            }
            EventFilterView::Any(filters) => {
                Self::Any(filters.into_iter().map(|f| f.into()).collect())
            }
            EventFilterView::And(left, right) => {
                Self::And(Box::new((*left).into()), Box::new((*right).into()))
            }
            EventFilterView::Or(left, right) => {
                Self::Or(Box::new((*left).into()), Box::new((*right).into()))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct EventView {
    pub event_id: EventID,
    pub type_tag: TypeTagView,
    pub event_data: StrView<Vec<u8>>,
    pub event_index: u64,
}

impl From<Event> for EventView {
    fn from(event: Event) -> Self {
        EventView {
            event_id: event.event_id,
            type_tag: event.type_tag.into(),
            event_data: StrView(event.event_data),
            event_index: event.event_index,
        }
    }
}

impl From<EventView> for Event {
    fn from(event: EventView) -> Self {
        Event {
            event_id: event.event_id,
            type_tag: event.type_tag.into(),
            event_data: event.event_data.0,
            event_index: event.event_index,
        }
    }
}
