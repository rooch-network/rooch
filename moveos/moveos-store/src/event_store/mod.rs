// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Error, Result};
// use anyhow::{bail, ensure, format_err, Result, Error};
use moveos_types::event::Event;
use moveos_types::h256::H256;
// use std::collections::BTreeMap;
use move_core_types::language_storage::TypeTag;
use moveos_types::event_filter::EventFilter;
use moveos_types::move_types::type_tag_match;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct EventStore {
    store: Arc<RwLock<HashMap<H256, Vec<Event>>>>,
    // store: Arc<RwLock<BTreeMap<H256, Vec<u8>>>>,
}

impl EventStore {
    /// Init EventStore with memory store, just for test
    pub fn new_with_memory_store() -> Self {
        Self {
            // store: Arc::new(RwLock::new(BTreeMap::new())),
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn save_events(&self, tx_hash: H256, events: Vec<Event>) -> Result<(), Error> {
        let mut locked = self.store.write();
        if locked.contains_key(&tx_hash) {
            let _old_events = locked.remove(&tx_hash);
        }
        // let raw_value = bcs::to_bytes(&events)?;
        locked.insert(tx_hash, events);
        Ok(())
    }

    pub fn get_events_by_tx_hash(&self, tx_hash: &H256) -> Result<Option<Vec<Event>>, Error> {
        let rw_locks = self.store.read();
        let result = rw_locks.get(tx_hash);
        // Ok(result.map(|raw_value| bcs::from_bytes(raw_value).ok().unwrap()))
        // Ok(result.map(|raw_value| raw_value.clone()))
        Ok(result.cloned())
    }

    pub fn multi_get_events_by_tx_hash(
        &self,
        tx_hashes: &[H256],
    ) -> Result<Option<Vec<Event>>, Error> {
        let mut result: Vec<Event> = Vec::new();
        for ev in tx_hashes
            .iter()
            .map(|tx_hash| self.get_events_by_tx_hash(tx_hash).unwrap())
            .collect::<Vec<_>>()
        {
            if Option::is_some(&ev) {
                result.append(&mut ev.unwrap());
            }
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub fn get_events_by_move_event_type(
        &self,
        move_event_type: &TypeTag,
    ) -> Result<Option<Vec<Event>>, Error> {
        let rw_locks = self.store.read();

        let mut result: Vec<Event> = Vec::new();
        for ev in rw_locks
            .iter()
            .map(|(_tx_hash, events)| {
                for event in events {
                    if type_tag_match(event.type_tag(), move_event_type) {
                        return Some(event.clone());
                        // result_events.push(event);
                    }
                }
                None
            })
            .collect::<Vec<_>>()
        {
            if Option::is_some(&ev) {
                // events.pop(&mut ev.unwrap());
                result.push(ev.unwrap());
            }
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    // TODO The complete event filter implementation depends on Indexer
    pub fn get_events_with_filter(&self, filter: EventFilter) -> Result<Option<Vec<Event>>, Error> {
        let result = match filter {
            // EventFilter::All(filters) => {
            //     return Err(anyhow!(
            //         "This type does not currently support filter combinations."
            //     ));
            // }
            EventFilter::Transaction(tx_hash) => self.get_events_by_tx_hash(&tx_hash)?,
            EventFilter::MoveEventType(move_event_type) => {
                self.get_events_by_move_event_type(&move_event_type)?
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
            EventFilter::BlockRange {
                from_block: _,
                to_block: _,
            } => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
            _ => {
                return Err(anyhow!(
                    "This type does not currently support filter combinations."
                ));
            }
        };

        Ok(result)
    }
}
