// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{schema::events, utils};
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::EventID;
use moveos_types::moveos_std::object::ObjectID;
use rooch_types::indexer::event::{IndexerEvent, IndexerEventID};
use std::str::FromStr;

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = events)]
pub struct StoredEvent {
    /// event handle id
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub event_handle_id: String,
    /// the number of messages that have been emitted to the path previously
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub event_seq: i64,
    /// the type of the event data
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub event_type: String,
    /// the data payload of the event
    #[diesel(sql_type = diesel::sql_types::Blob)]
    pub event_data: Vec<u8>,
    /// event index in the transaction events
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub event_index: i64,

    /// the hash of this transaction.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub tx_hash: String,
    /// the tx order of this transaction.
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,
    /// the rooch address of sender who emit the event
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub sender: String,

    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
}

impl From<IndexerEvent> for StoredEvent {
    fn from(event: IndexerEvent) -> Self {
        Self {
            event_handle_id: event.event_id.event_handle_id.to_string(),
            event_seq: event.event_id.event_seq as i64,
            event_type: utils::format_struct_tag(&event.event_type),
            event_data: event.event_data,
            event_index: event.indexer_event_id.event_index as i64,

            tx_hash: format!("{:?}", event.tx_hash),
            tx_order: event.indexer_event_id.tx_order as i64,
            sender: event.sender.to_hex_literal(),
            created_at: event.created_at as i64,
        }
    }
}

impl StoredEvent {
    pub fn try_into_indexer_event(&self) -> Result<IndexerEvent, anyhow::Error> {
        let event_handle_id = ObjectID::from_str(self.event_handle_id.as_str())?;
        let sender = AccountAddress::from_hex_literal(self.sender.as_str())?;
        let tx_hash = H256::from_str(self.tx_hash.as_str())?;
        let event_type = StructTag::from_str(self.event_type.as_str())?;

        let indexer_event = IndexerEvent {
            indexer_event_id: IndexerEventID::new(self.tx_order as u64, self.event_index as u64),
            event_id: EventID::new(event_handle_id, self.event_seq as u64),
            event_type,
            event_data: self.event_data.clone(),
            tx_hash,
            sender,
            created_at: self.created_at as u64,
        };
        Ok(indexer_event)
    }
}
