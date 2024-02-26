// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{EVENT_HANDLE_PREFIX_NAME, EVENT_PREFIX_NAME};
use anyhow::{anyhow, Result};
use move_core_types::language_storage::StructTag;
use moveos_types::moveos_std::event::{Event, EventHandle, EventID, TransactionEvent};
use moveos_types::moveos_std::object_id::ObjectID;
use raw_store::{derive_store, CodecKVStore, StoreInstance};
use std::cmp::min;
use std::collections::{HashMap, HashSet};

derive_store!(EventDBBaseStore, (ObjectID, u64), Event, EVENT_PREFIX_NAME);

derive_store!(
    EventHandleDBStore,
    ObjectID,
    EventHandle,
    EVENT_HANDLE_PREFIX_NAME
);

pub trait EventStore {
    fn save_events(&self, events: Vec<TransactionEvent>) -> Result<Vec<EventID>>;

    fn get_event(&self, event_id: EventID) -> Result<Option<Event>>;

    fn multi_get_events(&self, event_ids: Vec<EventID>) -> Result<Vec<Option<Event>>>;

    fn get_events_by_event_handle_id(
        &self,
        event_handle_id: &ObjectID,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<Event>>;

    fn get_events_by_event_handle_type(
        &self,
        event_handle_type: &StructTag,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<Event>>;
}

#[derive(Clone)]
pub struct EventDBStore {
    event_store: EventDBBaseStore,
    event_handle_store: EventHandleDBStore,
}

impl EventDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        EventDBStore {
            event_store: EventDBBaseStore::new(instance.clone()),
            event_handle_store: EventHandleDBStore::new(instance),
        }
    }

    fn get_event_handle(&self, event_handle_id: ObjectID) -> Result<Option<EventHandle>> {
        self.event_handle_store.kv_get(event_handle_id)
    }

    fn get_or_create_event_handle(&self, event_handle_type: &StructTag) -> Result<EventHandle> {
        let event_handle_id = EventHandle::derive_event_handle_id(event_handle_type);
        let event_handle = self.get_event_handle(event_handle_id.clone())?;
        if let Some(event_handle) = event_handle {
            return Ok(event_handle);
        }
        let event_handle = EventHandle::new(event_handle_id, 0);
        self.save_event_handle(event_handle.clone())?;
        Ok(event_handle)
    }

    fn save_event_handle(&self, event_handle: EventHandle) -> Result<()> {
        self.event_handle_store
            .put_all(vec![(event_handle.id.clone(), event_handle)])
    }

    pub fn save_events(&self, tx_events: Vec<TransactionEvent>) -> Result<Vec<EventID>> {
        let event_types = tx_events
            .iter()
            .map(|event| event.event_type.clone())
            .collect::<HashSet<_>>();
        let mut event_handles = event_types
            .into_iter()
            .map(|event_type| {
                let handle = self.get_or_create_event_handle(&event_type)?;
                Ok((event_type, handle))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        let mut event_ids = vec![];
        let events = tx_events
            .into_iter()
            .map(|tx_event| {
                let handle = event_handles
                    .get_mut(&tx_event.event_type)
                    .expect("Event handle must exist");
                let event_id = EventID::new(handle.id.clone(), handle.count);
                let event = Event::new(
                    event_id.clone(),
                    tx_event.event_type,
                    tx_event.event_data,
                    tx_event.event_index,
                );
                handle.count += 1;
                event_ids.push(event_id.clone());
                ((event_id.event_handle_id, event_id.event_seq), event)
            })
            .collect::<Vec<_>>();
        self.event_store.put_all(events)?;
        self.event_handle_store.put_all(
            event_handles
                .into_values()
                .map(|handle| (handle.id.clone(), handle))
                .collect::<Vec<_>>(),
        )?;
        Ok(event_ids)
    }

    pub fn get_event(&self, event_id: EventID) -> Result<Option<Event>> {
        let key = (event_id.event_handle_id, event_id.event_seq);
        self.event_store.kv_get(key)
    }

    pub fn multi_get_events(&self, event_ids: Vec<EventID>) -> Result<Vec<Option<Event>>> {
        let keys: Vec<_> = event_ids
            .into_iter()
            .map(|v| (v.event_handle_id, v.event_seq))
            .collect();
        self.event_store.multiple_get(keys)
    }

    /// Get events by event handle id
    /// The cursor is the previous last event seq
    /// So, do not include the result
    pub fn get_events_by_event_handle_id(
        &self,
        event_handle_id: &ObjectID,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<Event>> {
        let event_handle = self
            .get_event_handle(event_handle_id.clone())?
            .ok_or_else(|| {
                anyhow!(
                    "Can not find event handle by id: {}",
                    event_handle_id.to_string()
                )
            })?;
        let last_seq = event_handle.count;

        let ids = if descending_order {
            let start = cursor.unwrap_or(last_seq + 1);
            let end = if start >= limit { start - limit } else { 0 };
            (end..start).rev().collect::<Vec<_>>()
        } else {
            let start = match cursor {
                //The cursor do not include the result
                Some(cursor) => cursor + 1,
                //None means start from -1
                None => 0,
            };
            let end = min(start + limit, last_seq + 1);
            (start..end).collect::<Vec<_>>()
        };

        let event_ids = ids
            .into_iter()
            .map(|v| (EventID::new(event_handle_id.clone(), v)))
            .collect::<Vec<_>>();
        Ok(self
            .multi_get_events(event_ids)?
            .into_iter()
            .flatten()
            .collect())
    }

    pub fn get_events_by_event_handle_type(
        &self,
        event_handle_type: &StructTag,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<Event>> {
        let event_handle_id = EventHandle::derive_event_handle_id(event_handle_type);
        self.get_events_by_event_handle_id(&event_handle_id, cursor, limit, descending_order)
    }
}
