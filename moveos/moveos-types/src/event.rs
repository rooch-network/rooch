// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::ensure;
use std::str::FromStr;
// use move_bytecode_utils::module_cache::GetModule;
use move_core_types::account_address::AccountAddress;
// use move_core_types::identifier::IdentStr;
// use move_core_types::identifier::Identifier;
use move_core_types::language_storage::StructTag;
// use move_core_types::value::MoveStruct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use serde_with::Bytes;

// use crate::base_types::{ObjectID, AccountAddress, H256};
// use crate::{H256};
use crate::h256::H256;

/// A universal Rooch event type encapsulating different types of events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Transaction Info Id
    pub tx_id: H256,
    /// Consecutive per-tx counter assigned to this event.
    pub event_num: u64,
    /// Specific event type
    pub event: Event,
    /// Move event's json value
    pub parsed_json: Value,
    /// UTC timestamp in milliseconds since epoch (1/1/1970)
    pub timestamp: u64,
}
/// Unique ID of a Rooch Event, the ID is a combination of tx info id and event seq number,
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventID {
    pub tx_id: H256,
    pub event_seq: u64,
}

impl From<(H256, u64)> for EventID {
    fn from((tx_id, event_seq_number): (H256, u64)) -> Self {
        Self {
            tx_id,
            event_seq: event_seq_number,
        }
    }
}

impl From<EventID> for String {
    fn from(id: EventID) -> Self {
        format!("{:?}:{}", id.tx_id, id.event_seq)
    }
}

impl TryFrom<String> for EventID {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let values = value.split(':').collect::<Vec<_>>();
        ensure!(values.len() == 2, "Malformed EventID : {value}");
        Ok((H256::from_str(values[0])?, u64::from_str(values[1])?).into())
    }
}

impl EventEnvelope {
    pub fn new(
        timestamp: u64,
        tx_id: H256,
        event_num: u64,
        event: Event,
        move_struct_json_value: Value,
    ) -> Self {
        Self {
            timestamp,
            tx_id,
            event_num,
            event,
            parsed_json: move_struct_json_value,
        }
    }
}

/// Specific type of event
#[serde_as]
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct Event {
    // pub package_id: ObjectID,
    // pub transaction_module: Identifier,
    pub sender: AccountAddress,
    pub type_: StructTag,
    #[serde_as(as = "Bytes")]
    pub contents: Vec<u8>,
}

impl Event {
    pub fn new(
        // package_id: &AccountAddress,
        // module: &IdentStr,
        sender: AccountAddress,
        type_: StructTag,
        contents: Vec<u8>,
    ) -> Self {
        Self {
            // package_id: ObjectID::from(*package_id),
            // transaction_module: Identifier::from(module),
            sender,
            type_,
            contents,
        }
    }
    // pub fn move_event_to_move_struct(
    //     type_: &StructTag,
    //     contents: &[u8],
    //     resolver: &impl GetModule,
    // ) -> Result<MoveStruct> {
    //     let layout = MoveObject::get_layout_from_struct_tag(
    //         type_.clone(),
    //         ObjectFormatOptions::default(),
    //         resolver,
    //     )?;
    //     MoveStruct::simple_deserialize(contents, &layout).map_err(|e| {
    //         Error::ObjectSerializationError {
    //             error: e.to_string(),
    //         }
    //     })
    // }
}
