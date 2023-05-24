// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
// use anyhow::{bail, ensure, format_err, Result, Error};
use moveos_types::event::Event;
use moveos_types::h256::H256;
// use moveos_types::{event::Event, event::EventID};
// use std::collections::BTreeMap;
use parking_lot::RwLock;
use std::{
    // collections::{hash_map::Entry, HashMap},
    collections::HashMap,
    // convert::{TryFrom, TryInto},
    // iter::Peekable,
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct EventStore {
    // store: Arc<RwLock<BTreeMap<H256, Vec<Event>>>>,
    store: Arc<RwLock<HashMap<H256, Vec<u8>>>>,
}

impl EventStore {
    /// Init EventStore with memory store, just for test
    pub fn new_with_memory_store() -> Self {
        Self {
            // store: Arc::new(RwLock::new(BTreeMap::new())),
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn save_events(&self, txn_info_id: H256, events: Vec<Event>) -> Result<(), Error> {
        let mut locked = self.store.write();
        if locked.contains_key(&txn_info_id) {
            let _old_events = locked.remove(&txn_info_id);
        }
        let raw_value = bcs::to_bytes(&events)?;
        locked.insert(txn_info_id, raw_value);
        Ok(())
    }

    pub fn get_events(&self, txn_info_id: H256) -> Result<Option<Vec<Event>>, Error> {
        let rw_locks = self.store.read();
        let result = rw_locks.get(&txn_info_id);
        Ok(result.map(|raw_value| bcs::from_bytes(raw_value).ok().unwrap()))
    }
}
