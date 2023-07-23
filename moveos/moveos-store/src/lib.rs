// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
// use anyhow::{bail, format_err, Error, Result};
use once_cell::sync::Lazy;
use raw_store::{ColumnFamilyName, StoreInstance};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use crate::event_store::{EventDBStore, EventStore};
use crate::state_store::{StateDBStore};
use crate::transaction_store::{TransactionDBStore, TransactionStore};
use move_core_types::language_storage::TypeTag;
use moveos_types::event::{Event, EventID};
use moveos_types::event_filter::EventFilter;
use moveos_types::h256::H256;
use moveos_types::object::ObjectID;
use moveos_types::transaction::TransactionExecutionInfo;
use smt::NodeStore;

pub mod event_store;
pub mod state_store;
pub mod transaction_store;

// pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
pub const STATE_NODE_PREFIX_NAME: ColumnFamilyName = "state_node";
pub const TRANSACTION_PREFIX_NAME: ColumnFamilyName = "transaction";
pub const EVENT_PREFIX_NAME: ColumnFamilyName = "event";
pub const EVENT_INDEX_PREFIX_NAME: ColumnFamilyName = "event_index";

///db store use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
static VEC_PREFIX_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        STATE_NODE_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        EVENT_PREFIX_NAME,
        EVENT_INDEX_PREFIX_NAME,
    ]
});

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StoreMeta {
}

impl StoreMeta {
    pub fn get_column_family_names() -> &'static [ColumnFamilyName] {
            &VEC_PREFIX_NAME
    }
}

// #[derive(Clone)]
pub struct MoveOSStore {
    pub state_store: StateDBStore,
    pub event_store: EventDBStore,
    pub transaction_store: TransactionDBStore,
}
// // TODO: remove Arc<dyn Store>, we can clone Store directly.

impl MoveOSStore {
    // pub fn new_with_memory_store() -> Self {
    //     Self {
    //         state_store: StateDB::new_with_memory_store(),
    //         event_store: EventDB::new_with_memory_store(),
    //         transaction_store: TransactionDB::new_with_memory_store(),
    //     }
    // }

    pub fn new(instance: StoreInstance) -> Result<Self> {
        let store = Self {
            state_store: StateDBStore::new(instance.clone()),
            event_store: EventDBStore::new(instance.clone()),
            transaction_store: TransactionDBStore::new(instance.clone()),
        };
        Ok(store)
    }

    // pub fn get_node_store(&self) -> &NodeDBStore {
    //     &self.node_store
    // }

    pub fn get_event_store(&self) -> &EventDBStore {
        &self.event_store
    }

    pub fn get_transaction_store(&self) -> &TransactionDBStore {
        &self.transaction_store
    }

    pub fn get_state_store(&self) -> &StateDBStore {
        &self.state_store
    }
}

impl Display for MoveOSStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clone())
    }
}
impl Debug for MoveOSStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

// impl NodeStore for MoveOSStore {
//     fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>> {
//         self.state_store.node_store.get(*hash)
//     }
//
//     fn put(&self, key: H256, node: Vec<u8>) -> Result<()> {
//         self.state_store.node_store.put(key, node)
//     }
//
//     fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
//         let batch = CodecWriteBatch::new_puts(nodes.into_iter().collect());
//         self.state_store.node_store.write_batch(batch)
//     }
// }

impl NodeStore for MoveOSStore {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>> {
        self.state_store.node_store.get(hash)
    }

    fn put(&self, key: H256, node: Vec<u8>) -> Result<()> {
        self.state_store.node_store.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        self.state_store.node_store.write_nodes(nodes)
    }
}

impl EventStore for MoveOSStore {
    fn save_event(&self, event: Event) -> Result<()> {
        self.event_store.save_event(event)
    }

    fn save_events(&self, events: Vec<Event>) -> Result<()> {
        self.event_store.save_events(events)
    }

    fn get_event(&self, event_id: EventID) -> Result<Option<Event>> {
        self.event_store.get_event(event_id)
    }

    fn get_events_by_tx_hash(&self, tx_hash: &H256) -> Result<Vec<Event>> {
        self.event_store.get_events_by_tx_hash(tx_hash)
    }

    fn get_events_by_event_handle_id(
        &self,
        event_handle_id: &ObjectID,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Event>> {
        self.event_store
            .get_events_by_event_handle_id(event_handle_id, cursor, limit)
    }

    fn get_events_by_event_handle_type(&self, event_handle_type: &TypeTag) -> Result<Vec<Event>> {
        self.event_store
            .get_events_by_event_handle_type(event_handle_type)
    }

    fn get_events_with_filter(&self, filter: EventFilter) -> Result<Vec<Event>> {
        self.event_store.get_events_with_filter(filter)
    }
}

impl TransactionStore for MoveOSStore {
    fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) -> Result<()> {
        self.transaction_store.save_tx_exec_info(tx_exec_info)
    }

    fn get_tx_exec_info(&self, tx_hash: H256) -> Result<Option<TransactionExecutionInfo>> {
        self.transaction_store.get_tx_exec_info(tx_hash)
    }

    fn multi_get_tx_exec_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        self.transaction_store.multi_get_tx_exec_infos(tx_hashes)
    }
}

/// Moveos store define
pub trait Store: NodeStore + TransactionStore + EventStore + IntoSuper<dyn NodeStore> {
    // fn get_block_transaction_infos(
    //     &self,
    //     block_id: H256,
    // ) -> Result<Vec<RichTransactionInfo>, Error> {
    //     let txn_info_ids = self.get_block_txn_info_ids(block_id)?;
    //     let mut txn_infos = vec![];
    //     let txn_opt_infos = self.get_transaction_infos(txn_info_ids.clone())?;

    //     Ok(txn_infos)
    // }
    //
    // fn get_accumulator_store(
    //     &self,
    //     accumulator_type: AccumulatorStoreType,
    // ) -> Arc<dyn AccumulatorTreeStore>;
}

pub trait IntoSuper<Super: ?Sized> {
    fn as_super(&self) -> &Super;
    fn as_super_mut(&mut self) -> &mut Super;
    fn into_super(self: Box<Self>) -> Box<Super>;
    fn into_super_arc(self: Arc<Self>) -> Arc<Super>;
}

impl<'a, T: 'a + NodeStore> IntoSuper<dyn NodeStore + 'a> for T {
    fn as_super(&self) -> &(dyn NodeStore + 'a) {
        self
    }
    fn as_super_mut(&mut self) -> &mut (dyn NodeStore + 'a) {
        self
    }
    fn into_super(self: Box<Self>) -> Box<dyn NodeStore + 'a> {
        self
    }
    fn into_super_arc(self: Arc<Self>) -> Arc<dyn NodeStore + 'a> {
        self
    }
}

impl Store for MoveOSStore {
    // fn get_accumulator_store(
    //     &self,
    //     accumulator_type: AccumulatorStoreType,
    // ) -> Arc<dyn AccumulatorTreeStore> {
    //     match accumulator_type {
    //         AccumulatorStoreType::Block => Arc::new(self.block_accumulator_store.clone()),
    //         AccumulatorStoreType::Transaction => {
    //             Arc::new(self.transaction_accumulator_store.clone())
    //         }
    //     }
    // }
}
