// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::event_store::EventStore;
use crate::state_store::StateDB;
use crate::transaction_store::TransactionDB;

pub mod event_store;
pub mod state_store;
pub mod transaction_store;

pub struct MoveOSDB {
    pub state_store: StateDB,
    pub event_store: EventStore,
    pub transaction_store: TransactionDB,
}

impl MoveOSDB {
    pub fn new_with_memory_store() -> Self {
        Self {
            state_store: StateDB::new_with_memory_store(),
            event_store: EventStore::new_with_memory_store(),
            transaction_store: TransactionDB::new_with_memory_store(),
        }
    }

    pub fn get_state_store(&self) -> &StateDB {
        &self.state_store
    }

    pub fn get_event_store(&self) -> &EventStore {
        &self.event_store
    }

    pub fn get_transaction_store(&self) -> &TransactionDB {
        &self.transaction_store
    }
}
