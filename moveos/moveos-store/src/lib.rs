// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::event_store::EventStore;
use crate::state_store::StateDB;
// use std::sync::Arc;

pub mod event_store;
pub mod state_store;

// #[derive(Clone)]
pub struct MoveOSDB {
    pub state_store: StateDB,
    pub event_store: EventStore,
    // state_store: Arc<StateDB>,
    // event_store: Arc<EventStore>,
}

impl MoveOSDB {
    pub fn new_with_memory_store() -> Self {
        Self {
            state_store: StateDB::new_with_memory_store(),
            event_store: EventStore::new_with_memory_store(),
            // state_store: Arc::new(StateDB::new_with_memory_store()),
            // event_store: Arc::new(EventStore::new_with_memory_store()),
        }
    }

    pub fn get_state_store(&self) -> &StateDB {
        &self.state_store
    }

    pub fn get_event_store(&self) -> &EventStore {
        &self.event_store
    }
}
