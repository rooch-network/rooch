// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{EVENT_HANDLE_PREFIX_NAME, EVENT_INDEX_PREFIX_NAME, EVENT_PREFIX_NAME};
use anyhow::{anyhow, Result};
use move_core_types::language_storage::StructTag;
use moveos_types::event_filter::EventFilter;
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::{Event, EventHandle, EventID, TransactionEvent};
use moveos_types::moveos_std::object::ObjectID;
use raw_store::{derive_store, CodecKVStore, StoreInstance};
use std::cmp::min;
use std::collections::{HashMap, HashSet};

derive_store!(EventDBBaseStore, (ObjectID, u64), Event, EVENT_PREFIX_NAME);

//TODO the EventIndexDBStore is not used now, should remove it, and use the Indexer to get the event by tx_hash
derive_store!(
    EventIndexDBStore,
    (H256, u64),
    Event,
    EVENT_INDEX_PREFIX_NAME
);

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
    fn get_events_by_tx_hash(&self, tx_hash: &H256) -> Result<Vec<Event>>;

    fn get_events_by_event_handle_id(
        &self,
        event_handle_id: &ObjectID,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Event>>;

    fn get_events_by_event_handle_type(
        &self,
        event_handle_type: &StructTag,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Event>>;

    fn get_events_with_filter(&self, filter: EventFilter) -> Result<Vec<Event>>;
}

#[derive(Clone)]
pub struct EventDBStore {
    event_store: EventDBBaseStore,
    indexer_store: EventIndexDBStore,
    event_handle_store: EventHandleDBStore,
}

impl EventDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        EventDBStore {
            event_store: EventDBBaseStore::new(instance.clone()),
            indexer_store: EventIndexDBStore::new(instance.clone()),
            event_handle_store: EventHandleDBStore::new(instance),
        }
    }

    fn get_event_handle(&self, event_handle_id: ObjectID) -> Result<Option<EventHandle>> {
        self.event_handle_store.kv_get(event_handle_id)
    }

    fn get_or_create_event_handle(&self, event_handle_type: &StructTag) -> Result<EventHandle> {
        let event_handle_id = EventHandle::derive_event_handle_id(event_handle_type);
        let event_handle = self.get_event_handle(event_handle_id)?;
        if let Some(event_handle) = event_handle {
            return Ok(event_handle);
        }
        let event_handle = EventHandle::new(event_handle_id, 0);
        self.save_event_handle(event_handle.clone())?;
        Ok(event_handle)
    }

    fn save_event_handle(&self, event_handle: EventHandle) -> Result<()> {
        self.event_handle_store
            .put_all(vec![(event_handle.id, event_handle)])
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
                let event_id = EventID::new(handle.id, handle.count);
                let event = Event::new(
                    event_id,
                    tx_event.event_type,
                    tx_event.event_data,
                    tx_event.event_index,
                );
                handle.count += 1;
                event_ids.push(event_id);
                ((event_id.event_handle_id, event_id.event_seq), event)
            })
            .collect::<Vec<_>>();
        self.event_store.put_all(events)?;
        self.event_handle_store.put_all(
            event_handles
                .into_values()
                .map(|handle| (handle.id, handle))
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

    pub fn get_events_by_tx_hash(&self, tx_hash: &H256) -> Result<Vec<Event>> {
        let mut iter = self.indexer_store.iter()?;
        let seek_key = (*tx_hash, 0u64);
        iter.seek(bcs::to_bytes(&seek_key)?)
            .map_err(|e| anyhow::anyhow!("EventStore get_events_by_tx_hash seek: {:?}", e))?;
        let data: Vec<Event> = iter
            .filter_map(|item| {
                let ((tx_hash_key, _), event) = item.unwrap_or_else(|err| {
                    panic!("{}", format!("Get events by tx hash error, {:?}", err))
                });
                if tx_hash_key == *tx_hash {
                    Some(event)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Ok(data)
    }

    pub fn get_events_by_event_handle_id(
        &self,
        event_handle_id: &ObjectID,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Event>> {
        let event_handle = self.get_event_handle(*event_handle_id)?.ok_or_else(|| {
            anyhow!(
                "Can not find event handle by id: {}",
                event_handle_id.to_string()
            )
        })?;
        let last_seq = event_handle.count;
        let start = cursor.unwrap_or(0);
        let end = min(start + limit, last_seq);
        let event_ids = (start..end)
            .map(|v| (EventID::new(*event_handle_id, v)))
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
    ) -> Result<Vec<Event>> {
        let event_handle_id = EventHandle::derive_event_handle_id(event_handle_type);
        self.get_events_by_event_handle_id(&event_handle_id, cursor, limit)
    }

    // TODO The complete event filter implementation depends on Indexer
    pub fn get_events_with_filter(&self, filter: EventFilter) -> Result<Vec<Event>> {
        let result = match filter {
            EventFilter::All(_filters) => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
            // EventFilter::Transaction(tx_hash) => self.get_events_by_tx_hash(&tx_hash)?,
            EventFilter::MoveEventType(move_event_type) => {
                self.get_events_by_event_handle_type(&move_event_type, Some(0), 100)?
            }
            // EventFilter::Sender(_sender) => {
            //     return Err(anyhow!(
            //         "This type does not currently support filter combinations."
            //     ));
            // }
            EventFilter::MoveEventField { path: _, value: _ } => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
            // EventFilter::TimeRange {
            //     start_time: _,
            //     end_time: _,
            // } => {
            //     return Err(anyhow!(
            //         "This type does not currently support filter combinations."
            //     ));
            // },
            _ => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
        };

        Ok(result)
    }
}
