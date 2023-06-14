// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::object::ObjectID;
use anyhow::{ensure, Error, Result};
use move_core_types::account_address::AccountAddress;
use move_core_types::{
    language_storage::StructTag, language_storage::TypeTag, move_resource::MoveResource,
};
use schemars::JsonSchema;
// #[cfg(any(test, feature = "fuzzing"))]
// use rand::{rngs::OsRng, RngCore};
use crate::h256;
use crate::move_types::TypeInfo;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;

use crate::h256::H256;
use move_resource_viewer::AnnotatedMoveStruct;
use serde_with::serde_as;

/// A struct that represents a globally unique id for an Event stream that a user can listen to.
/// the Unique ID is a combination of event handle id and event seq number.
/// the ID is local to this particular fullnode and will be different from other fullnode.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
pub struct EventID {
    /// each event handle corresponds to a unique event handle id. event handler id equal to guid.
    pub event_handle_id: ObjectID,
    /// For expansion: The number of messages that have been emitted to the path previously
    pub event_seq: u64,
}

impl From<(ObjectID, u64)> for EventID {
    fn from((event_handle_id, event_seq): (ObjectID, u64)) -> Self {
        Self {
            event_handle_id,
            event_seq,
        }
    }
}

impl From<EventID> for String {
    fn from(id: EventID) -> Self {
        format!("{:?}:{}", id.event_handle_id, id.event_seq)
    }
}

impl TryFrom<String> for EventID {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let values = value.split(':').collect::<Vec<_>>();
        ensure!(values.len() == 2, "Malformed EventID : {value}");
        Ok((ObjectID::from_str(values[0])?, u64::from_str(values[1])?).into())
    }
}

impl FromStr for EventID {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EventID::try_from(s.to_string())
    }
}

impl std::fmt::Display for EventID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EventID[event handle id: {:?}, event seq: {}]",
            self.event_handle_id, self.event_seq,
        )
    }
}

impl EventID {
    /// Construct a new EventID.
    pub fn new(event_handle_id: ObjectID, event_seq: u64) -> Self {
        EventID {
            event_handle_id,
            event_seq,
        }
    }
}

// impl ser::Serialize for EventID {
//     fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
//     where
//         S: ser::Serializer,
//     {
//         if serializer.is_human_readable() {
//             self.to_string().serialize(serializer)
//         } else {
//             // In order to preserve the Serde data model and help analysis tools,
//             // make sure to wrap our value in a container with the same name
//             // as the original type.
//             serializer.serialize_newtype_struct("EventID", serde_bytes::Bytes::new(&self.0))
//         }
//     }
// }
//
// impl<'de> de::Deserialize<'de> for EventID {
//     fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
//     where
//         D: de::Deserializer<'de>,
//     {
//         if deserializer.is_human_readable() {
//             let s = <String>::deserialize(deserializer)?;
//             Self::from_str(s.as_str()).map_err(<D::Error as ::serde::de::Error>::custom)
//         } else {
//             // See comment in serialize.
//             #[derive(::serde::Deserialize)]
//             #[serde(rename = "EventID")]
//             struct Value<'a>(&'a [u8]);
//
//             let value = Value::deserialize(deserializer)?;
//             Self::try_from(value.0).map_err(<D::Error as ::serde::de::Error>::custom)
//         }
//     }
// }
//

/// Entry produced via a call to the `emit_event` builtin.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// The unique event_id that the event was emitted to
    pub event_id: EventID,
    // /// For expansion: The number of messages that have been emitted to the path previously
    // pub sequence_number: u64,
    /// The type of the data
    pub type_tag: TypeTag,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
    pub event_data: Vec<u8>,
    /// event index in the transaction events.
    pub event_index: u64,
}

impl Event {
    pub fn new(
        event_id: EventID,
        // sequence_number: u64,
        type_tag: TypeTag,
        event_data: Vec<u8>,
        event_index: u64,
    ) -> Self {
        Self {
            event_id,
            // sequence_number,
            type_tag,
            event_data,
            event_index,
        }
    }

    pub fn event_id(&self) -> &EventID {
        &self.event_id
    }

    pub fn event_data(&self) -> &[u8] {
        &self.event_data
    }

    pub fn decode_event<EventType: MoveResource + DeserializeOwned>(&self) -> Result<EventType> {
        bcs::from_bytes(self.event_data.as_slice()).map_err(Into::into)
    }

    pub fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }

    pub fn is<EventType: MoveResource>(&self) -> bool {
        self.type_tag == TypeTag::Struct(Box::new(EventType::struct_tag()))
    }

    pub fn hash(&self) -> H256 {
        h256::sha3_256_of(bcs::to_bytes(self).as_ref().unwrap())
    }
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Event {{ event_id: {:?}, type: {:?}, event_data: {:?} }}",
            self.event_id,
            // self.sequence_number,
            self.type_tag,
            hex::encode(&self.event_data)
        )
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A Rust representation of an Event Handle Resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventHandle {
    /// Number of events in the event stream.
    count: u64,
    /// each event handle corresponds to a unique event handle id
    event_handle_id: ObjectID,
    /// event handle create address
    sender: AccountAddress,
}

impl EventHandle {
    /// Constructs a new Event Handle
    pub fn new(event_handle_id: ObjectID, count: u64, sender: AccountAddress) -> Self {
        EventHandle {
            event_handle_id,
            count,
            sender,
        }
    }

    /// Return the event_id to where this event is stored in EventStore.
    pub fn event_handle_id(&self) -> &ObjectID {
        &self.event_handle_id
    }
    /// Return the counter for the handle
    pub fn count(&self) -> u64 {
        self.count
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn count_mut(&mut self) -> &mut u64 {
        &mut self.count
    }

    pub fn derive_event_handle_id(event_handle_type: StructTag) -> ObjectID {
        let type_info = TypeInfo::new(
            event_handle_type.address,
            event_handle_type.module,
            event_handle_type.name,
        );
        let event_handle_hash = h256::sha3_256_of(bcs::to_bytes(&type_info).unwrap().as_ref());
        AccountAddress::try_from(event_handle_hash.as_bytes())
            .unwrap()
            .into()
    }

    // #[cfg(any(test, feature = "fuzzing"))]
    // /// Create a random event handle for testing
    // pub fn random_handle(count: u64) -> Self {
    //     Self {
    //         event_id: EventID::random(),
    //         count,
    //     }
    // }

    // /// Derive a unique handle by using an AccountAddress and a counter.
    // pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
    //     Self {
    //         event_id: EventID::new_from_address(addr, salt),
    //         count: 0,
    //     }
    // }
}

// #[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
// #[derive(Clone, Debug)]
#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct MoveOSEvent {
    pub event: Event,
    /// Sender's address.
    pub sender: AccountAddress,
    /// Transaction hash
    pub tx_hash: Option<H256>,
    /// UTC timestamp in milliseconds since epoch (1/1/1970)
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[schemars(with = "Option<u64>")]
    // #[serde_as(as = "Option<u64>")]
    pub timestamp_ms: Option<u64>,
    // block height
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[schemars(with = "Option<u64>")]
    // #[serde_as(as = "Option<u64>")]
    // pub block_height: Option<u64>,
}

impl MoveOSEvent {
    pub fn new(
        event: Event,
        tx_hash: Option<H256>,
        timestamp_ms: Option<u64>,
        // block_height: Option<u64>,
    ) -> Self {
        let sender = AccountAddress::ZERO;

        MoveOSEvent {
            event,
            sender,
            tx_hash,
            timestamp_ms,
            // block_height,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedMoveOSEvent {
    pub event: Event,
    pub sender: AccountAddress,
    pub tx_hash: Option<H256>,
    // pub event_data: Vec<u8>,
    pub timestamp_ms: Option<u64>,
    // pub block_height: Option<u64>,
    pub parsed_event_data: AnnotatedMoveStruct,
}

impl AnnotatedMoveOSEvent {
    pub fn new(
        event: Event,
        parsed_event_data: AnnotatedMoveStruct,
        tx_hash: Option<H256>,
        timestamp_ms: Option<u64>,
        // block_height: Option<u64>,
    ) -> Self {
        let sender = AccountAddress::ZERO;
        AnnotatedMoveOSEvent {
            event,
            sender,
            tx_hash,
            parsed_event_data,
            timestamp_ms,
            // block_height,
        }
    }
}
