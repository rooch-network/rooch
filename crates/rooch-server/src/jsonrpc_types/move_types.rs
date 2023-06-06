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
use moveos_types::h256::H256;
use moveos_types::move_types::parse_module_id;
use moveos_types::{
    event::EventID,
    state::{AnnotatedState, State},
};
use moveos_types::{
    event_filter::MoveOSEvent,
    move_string::{MoveAsciiString, MoveString},
    state::MoveState,
};
use moveos_types::{
    move_types::FunctionId,
    object::{AnnotatedObject, ObjectID},
    transaction::FunctionCall,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

pub type ModuleIdView = StrView<ModuleId>;
pub type TypeTagView = StrView<TypeTag>;
pub type StructTagView = StrView<StructTag>;
pub type FunctionIdView = StrView<FunctionId>;

impl_str_view_for! {TypeTag StructTag FunctionId}
// impl_str_view_for! {TypeTag StructTag ModuleId FunctionId}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnnotatedMoveValueView {
    U8(u8),
    ///u64, u128, U256 is too large to be serialized in json
    /// so we use string to represent them
    U64(StrView<u64>),
    U128(StrView<u128>),
    Bool(bool),
    Address(StrView<AccountAddress>),
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
    pub owner: AccountAddress,
    pub value: AnnotatedMoveStructView,
}

impl From<AnnotatedObject> for AnnotatedObjectView {
    fn from(origin: AnnotatedObject) -> Self {
        Self {
            id: origin.id,
            owner: origin.owner,
            value: origin.value.into(),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<H256>,
    /// Sender's address.
    pub sender: AccountAddress,
    pub event_data: StrView<Vec<u8>>,
    /// Parsed json value of the event data
    // pub parsed_event_data: Value,
    pub parsed_event_data: AnnotatedMoveStructView,
    pub type_tag: TypeTagView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_index: Option<u32>,
    pub event_id: EventID,
}

impl From<MoveOSEvent> for EventView {
    fn from(event: MoveOSEvent) -> Self {
        EventView {
            tx_hash: event.tx_hash,
            sender: AccountAddress::ZERO, //Reserved as an extension field
            event_data: StrView(event.event_data.to_vec()),
            parsed_event_data: event.parsed_event_data.into(),
            type_tag: event.type_tag.clone().into(),
            event_index: Some(event.event_index),
            event_id: event.event_id,
        }
    }
}

// TODO:
pub struct TransactionView {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateView {
    pub value: StrView<Vec<u8>>,
    pub value_type: TypeTagView,
}

impl From<State> for StateView {
    fn from(state: State) -> Self {
        Self {
            value: StrView(state.value),
            value_type: state.value_type.into(),
        }
    }
}

impl From<StateView> for State {
    fn from(state: StateView) -> Self {
        Self {
            value: state.value.0,
            value_type: state.value_type.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnnotatedStateView {
    pub state: StateView,
    pub move_value: AnnotatedMoveValueView,
}

impl From<AnnotatedState> for AnnotatedStateView {
    fn from(state: AnnotatedState) -> Self {
        Self {
            state: state.state.into(),
            move_value: state.move_value.into(),
        }
    }
}

//TODO Is it need to convert the AnnotatedStateView back to AnnotatedState?
//If not, please remove this code. Otherwise, it needs to be fixed. include TryFrom<AnnotatedMoveValueView> for AnnotatedMoveValue
// impl TryFrom<AnnotatedStateView> for AnnotatedState {
//     type Error = anyhow::Error;

//     fn try_from(value: AnnotatedStateView) -> Result<Self, Self::Error> {
//         Ok(Self {
//             state: value.state.into(),
//             move_value: value.move_value.try_into()?,
//         })
//     }
// }
