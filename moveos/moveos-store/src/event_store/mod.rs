// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use move_core_types::language_storage::TypeTag;
use moveos_types::event_filter::EventFilter;
use moveos_types::h256::H256;
use moveos_types::move_types::type_tag_match;
use moveos_types::moveos_std::event::{Event, EventID};
use moveos_types::moveos_std::object::ObjectID;

use crate::{EVENT_INDEX_PREFIX_NAME, EVENT_PREFIX_NAME};
use raw_store::{derive_store, CodecKVStore, StoreInstance};

derive_store!(EventDBBaseStore, (ObjectID, u64), Event, EVENT_PREFIX_NAME);

derive_store!(
    EventIndexDBStore,
    (H256, u64),
    Event,
    EVENT_INDEX_PREFIX_NAME
);

pub trait EventStore {
    fn save_event(&self, event: Event) -> Result<()>;

    fn save_events(&self, events: Vec<Event>) -> Result<()>;

    fn get_event(&self, event_id: EventID) -> Result<Option<Event>>;

    fn multi_get_events(&self, event_ids: Vec<EventID>) -> Result<Vec<Option<Event>>>;
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

impl EventDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        EventDBStore {
            event_store: EventDBBaseStore::new(instance.clone()),
            indexer_store: EventIndexDBStore::new(instance),
        }
    }

    pub fn save_event(&self, event: Event) -> Result<()> {
        let key = (event.event_id.event_handle_id, event.event_id.event_seq);
        self.event_store.kv_put(key, event)
    }

    pub fn save_events(&self, events: Vec<Event>) -> Result<()> {
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
        //  will not cross the boundary even if the size exceeds the storage capacity,
        let start = cursor.unwrap_or(0);
        let end = start + limit;
        let mut iter = self.event_store.iter()?;
        let seek_key = (*event_handle_id, start);
        iter.seek(bcs::to_bytes(&seek_key)?).map_err(|e| {
            anyhow::anyhow!("EventStore get_events_by_event_handle_id seek: {:?}", e)
        })?;

        let data: Vec<Event> = iter
            .filter_map(|item| {
                let ((handle_id, event_seq), event) = item.unwrap_or_else(|err| {
                    panic!(
                        "{}",
                        format!("Get events by event handle id error, {:?}", err)
                    )
                });
                if Option::is_some(&cursor) {
                    if handle_id == *event_handle_id && (event_seq > start && event_seq <= end) {
                        return Some(event);
                    }
                } else if handle_id == *event_handle_id && (event_seq >= start && event_seq < end) {
                    return Some(event);
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
        let mut iter = self.event_store.iter()?;
        //TODO choose the right seek key to optimize performance
        iter.seek_to_first();
        let data: Vec<_> = iter
            .filter_map(|item| {
                let ((_event_handle_id, _event_seq), event) = item.unwrap_or_else(|err| {
                    panic!(
                        "{}",
                        format!("Get events by event handle type error, {:?}", err)
                    )
                });
                if type_tag_match(event.type_tag(), event_handle_type) {
                    Some(event)
                } else {
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
            // EventFilter::Transaction(tx_hash) => self.get_events_by_tx_hash(&tx_hash)?,
            EventFilter::MoveEventType(move_event_type) => {
                self.get_events_by_event_handle_type(&move_event_type)?
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
