// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Error, Result};
use move_core_types::language_storage::TypeTag;
use moveos_types::event::{Event, EventID};
use moveos_types::event_filter::EventFilter;
use moveos_types::h256::H256;
use moveos_types::move_types::type_tag_match;
use moveos_types::object::ObjectID;
use parking_lot::RwLock;
use std::{collections::BTreeMap, sync::Arc};

use crate::{EVENT_INDEX_PREFIX_NAME, EVENT_PREFIX_NAME};
use raw_store::{derive_store, CodecKVStore, CodecWriteBatch, StoreInstance};

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
        BlockStorage {
            event_store: EventDBBaseStore::new(instance.clone()),
            indexer_store: EventIndexDBStore::new(instance.clone()),
        }
    }

    pub fn save_event(&self, event: Event) -> Result<(), Error> {
        // let mut locked = self.event_store.write();
        // let key = (event.event_id.event_handle_id, event.event_id.event_seq);
        // locked.insert(key, event);
        // Ok(())
        let key = (event.event_id.event_handle_id, event.event_id.event_seq);
        self.event_store.put(key, event)
    }

    pub fn save_events(&self, events: Vec<Event>) -> Result<(), Error> {
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

    pub fn get_event(&self, event_id: EventID) -> Result<Option<Event>, Error> {
        // let rw_locks = self.event_store.read();
        // let key = (event_id.event_handle_id, event_id.event_seq);
        // let result = rw_locks.get(&key);
        // Ok(result.cloned())
        let key = (event_id.event_handle_id, event_id.event_seq);
        self.event_store.get(key)
    }

    //TODO implement event indexer for query by tx hash
    pub fn get_events_by_tx_hash(&self, tx_hash: &H256) -> Result<Vec<Event>, Error> {
        // let rw_locks = self.indexer_store.read();
        // let data = rw_locks
        //     .iter()
        //     .filter(|((tx_hash_key, _), _)| *tx_hash_key == *tx_hash)
        //     .map(|(_, e)| e.clone())
        //     .collect::<Vec<_>>();
        // Ok(data)

        let mut iter = self.indexer_store.iter()?;

        // for item in iter {
        //     let (k, v) = item?;
        // }
        let data: Vec<Event> = iter
            .filter(|((tx_hash_key, _), _)| *tx_hash_key == *tx_hash)
            .map(|(_, e)| e.clone())
            .collect::<Vec<_>>();
        Ok(data)

        // iter.skip_to(&(tx_seq, event_seq))?
        //     .take(limit)
        //     .map(|((_, event_seq), (digest, tx_digest, time))| {
        //         (digest, tx_digest, event_seq, time)
        //     })
        //     .collect()

        // Ok(vec![])
    }

    pub fn get_events_by_event_handle_id(
        &self,
        event_handle_id: &ObjectID,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Event>, Error> {
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
        let mut iter = self.event_store.iter()?;

        let data: Vec<Event> = iter
            .filter(|((handle_id, event_seq), _)| {
                if Option::is_some(&cursor) {
                    *handle_id == *event_handle_id && (*event_seq > start && *event_seq <= end)
                } else {
                    *handle_id == *event_handle_id && (*event_seq >= start && *event_seq < end)
                }
            })
            .map(|(_, e)| e.clone())
            .collect::<Vec<_>>();
        Ok(data)
    }

    pub fn get_events_by_event_handle_type(
        &self,
        event_handle_type: &TypeTag,
    ) -> Result<Vec<Event>, Error> {
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

        let mut iter = self.event_store.iter()?;

        let data: Vec<Event> = iter
            .filter_map(|((_event_handle_id, _event_seq), event)| {
                if type_tag_match(event.type_tag(), event_handle_type) {
                    return Some(event.clone());
                }
                None
            })
            .collect::<Vec<_>>();
        Ok(data)
    }

    // TODO The complete event filter implementation depends on Indexer
    pub fn get_events_with_filter(&self, filter: EventFilter) -> Result<Vec<Event>, Error> {
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
