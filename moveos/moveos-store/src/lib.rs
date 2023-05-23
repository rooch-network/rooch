// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::event_store::EventStore;
use anyhow::Result;
use std::sync::Arc;

pub mod event_store;
pub mod state_store;

#[derive(Clone)]
pub struct RoochDB {
    // event_store: EventStore,
    event_store: Arc<EventStore>,
}

impl RoochDB {
    pub fn new_with_memory_store() -> Result<Self> {
        let store = Self {
            // event_store: EventStore::new_with_memory_store(),
            event_store: Arc::new(EventStore::new_with_memory_store()),
        };
        Ok(store)
    }

    pub fn get_event_store(&self) -> Arc<EventStore> {
        self.event_store.clone()
    }
}
