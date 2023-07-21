// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use move_core_types::language_storage::TypeTag;
use moveos_types::event::{Event, EventID};
use moveos_types::event_filter::EventFilter;
use moveos_types::h256::H256;
use moveos_types::move_types::type_tag_match;
use moveos_types::object::ObjectID;
// use std::{collections::BTreeMap, sync::Arc};

use crate::{EVENT_INDEX_PREFIX_NAME, EVENT_PREFIX_NAME};
use raw_store::{derive_store, CodecKVStore, StoreInstance};

derive_store!(EventDBBaseStore, (ObjectID, u64), Event, EVENT_PREFIX_NAME);

derive_store!(
    EventIndexDBStore,
    (H256, u64),
    Event,
    EVENT_INDEX_PREFIX_NAME
);

// pub struct EventTxCombinaorID {
//     pub tx_hash: H256,
//     pub event_index: u64,
// }

pub trait EventStore {
    fn save_event(&self, event: Event) -> Result<()>;

    fn save_events(&self, events: Vec<Event>) -> Result<()>;

    fn get_event(&self, event_id: EventID) -> Result<Option<Event>>;

    fn get_events_by_tx_hash(&self, tx_hash: &H256) -> Result<Vec<Event>>;

    fn get_events_by_event_handle_id(
        &self,
        event_handle_id: &ObjectID,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Event>>;

    fn get_events_by_event_handle_type(&self, event_handle_type: &TypeTag) -> Result<Vec<Event>>;

    fn get_events_with_filter(&self, filter: EventFilter) -> Result<Vec<Event>>;
}

#[derive(Clone)]
pub struct EventDBStore {
    event_store: EventDBBaseStore,
    indexer_store: EventIndexDBStore,
}

// #[derive(Debug)]
// pub struct EventDB {
//     indexer_store: Arc<RwLock<BTreeMap<(H256, u64), Event>>>,
//     store: Arc<RwLock<BTreeMap<(ObjectID, u64), Event>>>,
// }

impl EventDBStore {
    /// Init EventDB with memory store, just for test
    // pub fn new_with_memory_store() -> Self {
    //     Self {
    //         store: Arc::new(RwLock::new(BTreeMap::new())),
    //         indexer_store: Arc::new(RwLock::new(BTreeMap::new())),
    //     }
    // }

    pub fn new(instance: StoreInstance) -> Self {
        EventDBStore {
            event_store: EventDBBaseStore::new(instance.clone()),
            indexer_store: EventIndexDBStore::new(instance.clone()),
        }
    }

    pub fn save_event(&self, event: Event) -> Result<()> {
        // let mut locked = self.event_store.write();
        // let key = (event.event_id.event_handle_id, event.event_id.event_seq);
        // locked.insert(key, event);
        // Ok(())
        let key = (event.event_id.event_handle_id, event.event_id.event_seq);
        self.event_store.kv_put(key, event)
    }

    pub fn save_events(&self, events: Vec<Event>) -> Result<()> {
        // let mut locked = self.event_store.write();
        // let data = events
        //     .into_iter()
        //     .map(|event| {
        //         (
        //             (event.event_id.event_handle_id, event.event_id.event_seq),
        //             event,
        //         )
        //     })
        //     .collect::<Vec<_>>();
        // locked.extend(data);
        // Ok(())

        self.event_store.put_all(
            events
                .into_iter()
                .map(|event| {
                    (
                        (event.event_id.event_handle_id, event.event_id.event_seq),
                        event,
                    )
                })
                .collect(),
        )
    }

    pub fn get_event(&self, event_id: EventID) -> Result<Option<Event>> {
        // let rw_locks = self.event_store.read();
        // let key = (event_id.event_handle_id, event_id.event_seq);
        // let result = rw_locks.get(&key);
        // Ok(result.cloned())
        let key = (event_id.event_handle_id, event_id.event_seq);
        self.event_store.kv_get(key)
    }

    pub fn get_events_by_tx_hash(&self, tx_hash: &H256) -> Result<Vec<Event>> {
        // let rw_locks = self.indexer_store.read();
        // let data = rw_locks
        //     .iter()
        //     .filter(|((tx_hash_key, _), _)| *tx_hash_key == *tx_hash)
        //     .map(|(_, e)| e.clone())
        //     .collect::<Vec<_>>();
        // Ok(data)

        let iter = self.indexer_store.iter()?;
        let data: Vec<Event> = iter
            .filter_map(|item| {
                let ((tx_hash_key, _), event) = item.unwrap_or_else(|_| panic!("Get item from store shoule hava a value."));
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
        // //  will not cross the boundary even if the size exceeds the storage capacity,
        // let start = cursor.unwrap_or(0);
        // let end = start + limit;
        // let rw_locks = self.event_store.read();
        // let data = rw_locks
        //     .iter()
        //     .filter(|((handle_id, event_seq), _)| {
        //         if Option::is_some(&cursor) {
        //             *handle_id == *event_handle_id && (*event_seq > start && *event_seq <= end)
        //         } else {
        //             *handle_id == *event_handle_id && (*event_seq >= start && *event_seq < end)
        //         }
        //     })
        //     .map(|(_, e)| e.clone())
        //     .collect::<Vec<_>>();
        // Ok(data)

        //  will not cross the boundary even if the size exceeds the storage capacity,
        let start = cursor.unwrap_or(0);
        let end = start + limit;
        let iter = self.event_store.iter()?;

        let data: Vec<Event> = iter
            .filter_map(|item| {
                let ((handle_id, event_seq), event) = item.unwrap_or_else(|_| panic!("Get item from store shoule hava a value."));
                if Option::is_some(&cursor) {
                    if handle_id == *event_handle_id && (event_seq > start && event_seq <= end) {
                        return Some(event)
                    }
                } else {
                    if handle_id == *event_handle_id && (event_seq >= start && event_seq < end) {
                        return Some(event)
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        Ok(data)
    }

    pub fn get_events_by_event_handle_type(
        &self,
        event_handle_type: &TypeTag,
    ) -> Result<Vec<Event>> {
        // let rw_locks = self.event_store.read();
        //
        // let data = rw_locks
        //     .iter()
        //     .filter_map(|((_event_handle_id, _event_seq), event)| {
        //         if type_tag_match(event.type_tag(), event_handle_type) {
        //             return Some(event.clone());
        //         }
        //         None
        //     })
        //     .collect::<Vec<_>>();
        // Ok(data)

        let iter = self.event_store.iter()?;
        let data: Vec<Event> = iter
            .filter_map(|item| {
                let ((_event_handle_id, _event_seq), event) = item.unwrap_or_else(|_| panic!("Get item from store shoule hava a value."));
                if type_tag_match(event.type_tag(), event_handle_type) {
                    Some(event)
                }else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Ok(data)
    }

    // TODO The complete event filter implementation depends on Indexer
    pub fn get_events_with_filter(&self, filter: EventFilter) -> Result<Vec<Event>> {
        let result = match filter {
            EventFilter::All(_filters) => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
            EventFilter::Transaction(tx_hash) => self.get_events_by_tx_hash(&tx_hash)?,
            EventFilter::MoveEventType(move_event_type) => {
                self.get_events_by_event_handle_type(&move_event_type)?
            }
            EventFilter::Sender(_sender) => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
            EventFilter::MoveEventField { path: _, value: _ } => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
            EventFilter::TimeRange {
                start_time: _,
                end_time: _,
            } => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
            // EventFilter::BlockRange {
            //     from_block: _,
            //     to_block: _,
            // } => {
            //     return Err(anyhow!(
            //         "This type does not currently support filter combinations."
            //     ));
            // }
            _ => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
        };

        Ok(result)
    }
}
