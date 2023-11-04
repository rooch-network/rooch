// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::events;
use crate::types::IndexedEvent;
use diesel::prelude::*;

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
    pub struct_tag: String,
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
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
}

impl From<IndexedEvent> for StoredEvent {
    fn from(event: IndexedEvent) -> Self {
        Self {
            event_handle_id: event.event_handle_id.to_string(),
            event_seq: event.event_seq as i64,
            struct_tag: event.struct_tag.to_canonical_string(),
            event_data: event.event_data,
            event_index: event.event_index as i64,

            // TODO use tx_hash: StrView(event.tx_hash) ?
            tx_hash: event.tx_hash.to_string(),
            tx_order: event.tx_order as i64,
            sender: event.sender.to_string(),

            created_at: event.created_at as i64,
            updated_at: event.updated_at as i64,
        }
    }
}
