// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::h256;
use crate::moveos_std::object_id::ObjectID;
use crate::state::MoveStructType;
use anyhow::{ensure, Error, Result};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;
use move_core_types::{ident_str, language_storage::StructTag, language_storage::TypeTag};
use move_resource_viewer::AnnotatedMoveStruct;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;

use crate::h256::H256;
use crate::module_binding::{ModuleBinding, MoveFunctionCaller};
use crate::moveos_std::tx_context::TxContext;
use crate::transaction::FunctionCall;

use super::object_id;

/// Rust bindings for MoveosStd event module
pub struct EventModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> EventModule<'a> {
    pub const GET_EVENT_HANDLE_FUNCTION_NAME: &'static IdentStr = ident_str!("get_event_handle");

    pub fn get_event_handle(
        &self,
        event_handle_type: StructTag,
    ) -> Result<(ObjectID, AccountAddress, u64)> {
        let ctx = TxContext::zero();
        let call = FunctionCall::new(
            Self::function_id(Self::GET_EVENT_HANDLE_FUNCTION_NAME),
            vec![TypeTag::Struct(Box::new(event_handle_type))],
            vec![],
        );

        let result = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map_err(|e| anyhow::anyhow!("Call get event handle error:{}", e))?;

        let event_handle_id = match result.get(0) {
            Some(value) => bcs::from_bytes::<ObjectID>(&value.value)?,
            None => return Err(anyhow::anyhow!("Event handle should have event handle id")),
        };
        let sender = match result.get(1) {
            Some(value) => bcs::from_bytes::<AccountAddress>(&value.value)?,
            None => return Err(anyhow::anyhow!("Event handle should have sender")),
        };
        let event_seq = match result.get(2) {
            Some(value) => bcs::from_bytes::<u64>(&value.value)?,
            None => return Err(anyhow::anyhow!("Event handle should have event seq")),
        };

        Ok((event_handle_id, sender, event_seq))
    }
}

impl<'a> ModuleBinding<'a> for EventModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("event");
    const MODULE_ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

/// A struct that represents a globally unique id for an Event stream that a user can listen to.
/// the Unique ID is a combination of event handle id and event seq number.
/// the ID is local to this particular fullnode and will be different from other fullnode.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
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

/// Entry produced via a call to the `emit_event` builtin.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionEvent {
    /// The type of the data
    pub event_type: StructTag,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
    pub event_data: Vec<u8>,
    /// event index in the transaction events.
    pub event_index: u64,
}

impl TransactionEvent {
    pub fn new(event_type: StructTag, event_data: Vec<u8>, event_index: u64) -> Self {
        Self {
            event_type,
            event_data,
            event_index,
        }
    }
}

/// The Event type in the event store
/// We generate the EventID in the event store, not in the event module.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// The unique event_id that the event was emitted to
    pub event_id: EventID,
    /// The type of the data
    pub event_type: StructTag,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
    pub event_data: Vec<u8>,
    /// event index in the transaction events.
    pub event_index: u64,
}

impl Event {
    pub fn new(
        event_id: EventID,
        event_type: StructTag,
        event_data: Vec<u8>,
        event_index: u64,
    ) -> Self {
        Self {
            event_id,
            event_type,
            event_data,
            event_index,
        }
    }

    pub fn new_with_event_id(event_id: EventID, transaction_event: TransactionEvent) -> Self {
        Self {
            event_id,
            event_type: transaction_event.event_type,
            event_data: transaction_event.event_data,
            event_index: transaction_event.event_index,
        }
    }

    pub fn event_id(&self) -> &EventID {
        &self.event_id
    }

    pub fn event_data(&self) -> &[u8] {
        &self.event_data
    }

    pub fn decode_event<EventType: MoveStructType + DeserializeOwned>(&self) -> Result<EventType> {
        bcs::from_bytes(self.event_data.as_slice()).map_err(Into::into)
    }

    pub fn event_type(&self) -> &StructTag {
        &self.event_type
    }

    pub fn is<EventType: MoveStructType>(&self) -> bool {
        self.event_type == EventType::struct_tag()
    }

    /// The event hashs of the transaction will be collect to build the transaction merkle tree root.
    /// The event hash is the hash of the event data, does not include other fields.
    pub fn hash(&self) -> H256 {
        h256::sha3_256_of(&self.event_data)
    }
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Event {{ event_id: {:?}, type: {:?}, event_data: {:?} }}",
            self.event_id,
            self.event_type,
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
    /// each event handle corresponds to a unique event handle id
    pub id: ObjectID,
    /// Number of events in the event stream.
    pub count: u64,
}

impl EventHandle {
    /// Constructs a new Event Handle
    pub fn new(id: ObjectID, count: u64) -> Self {
        EventHandle { id, count }
    }

    /// Return the event_id to where this event is stored in EventDB.
    pub fn id(&self) -> &ObjectID {
        &self.id
    }
    /// Return the counter for the handle
    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn count_mut(&mut self) -> &mut u64 {
        &mut self.count
    }

    pub fn derive_event_handle_id(event_handle_type: &StructTag) -> ObjectID {
        object_id::named_object_id(event_handle_type)
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedEvent {
    pub event: Event,
    pub decoded_event_data: AnnotatedMoveStruct,
}

impl AnnotatedEvent {
    pub fn new(event: Event, decoded_event_data: AnnotatedMoveStruct) -> Self {
        AnnotatedEvent {
            event,
            decoded_event_data,
        }
    }
}
