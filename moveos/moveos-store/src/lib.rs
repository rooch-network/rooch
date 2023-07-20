// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// use crate::accumulator::{
//     AccumulatorStore, BlockAccumulatorStore, TransactionAccumulatorStore,
// };
use crate::event::EventStore;
use crate::state_node::StateStore;
use raw_store::{CodecKVStore, CodecWriteBatch, ColumnFamilyName, StoreInstance};
use crate::transaction::TransactionStore;
use anyhow::{bail, format_err, Error, Result};
// use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;


// use crate::db_store::DBStore;
use crate::event_store::{EventDB, EventDBStore};
use crate::state_store::{NodeDBStore, StateDB, StateDBStore};
use crate::transaction_store::{TransactionDB, TransactionDBStore, TransactionStore};
use base_store::rocks::default_db_options;
use std::path::PathBuf;
use moveos_types::h256::H256;
use smt::NodeStore;

pub mod event_store;
pub mod state_store;
pub mod transaction_store;


pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
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
        EVENT_INDEX_PREFIX_NAME
    ]
});

pub trait EventStore {
    /// Save events by key `txn_info_id`.
    /// As txn_info has accumulator root of events, so there is a one-to-one mapping.
    fn save_events(
        &self,
        txn_info_id: H256,
        events: Vec<Event>,
    ) -> Result<()>;

    /// Get events by `txn_info_id`.
    /// If the txn_info_id does not exists in the store, return `None`.
    /// NOTICE: *don't exists* is different with *no events produced*.
    fn get_events(&self, txn_info_id: H256) -> Result<Option<Vec<Event>>>;
}

// pub trait TransactionStore {
//     fn get_transaction(&self, txn_hash: H256) -> Result<Option<Transaction>>;
//     fn save_transaction(&self, txn_info: Transaction) -> Result<()>;
//     fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<()>;
//     fn get_transactions(&self, txn_hash_vec: Vec<H256>) -> Result<Vec<Option<Transaction>>>;
// }


#[derive(Clone)]
pub struct MoveOSBaseStore {
    // pub state_store: NodeDBStore,
    pub node_store: NodeDBStore,
    pub event_store: EventDBStore,
    pub transaction_store: TransactionDBStore,
}

impl MoveOSBaseStore {
    pub fn new(instance: StoreInstance) -> Result<Self> {
        let store = Self {
            node_store: NodeDBStore::new(instance.clone()),
            event_store: EventDBStore::new(instance.clone()),
            transaction_store: TransactionDBStore::new(instance.clone()),
        };
        Ok(store)
    }

    pub fn get_node_store(&self) -> &NodeDBStore {
        &self.node_store
    }

    pub fn get_event_store(&self) -> &EventDBStore {
        &self.event_store
    }

    pub fn get_transaction_store(&self) -> &TransactionDBStore {
        &self.transaction_store
    }
}

#[derive(Clone)]
pub struct MoveOSStore {
    pub base_store: MoveOSBaseStore,
    pub state_store: StateDB,
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

    pub fn new_with_db_store(base_store: MoveOSBaseStore) -> Self {
        let state_store = StateDB::new_with_db_store(Arc::new(base_store.clone()));
        Self {
            base_store,
            state_store,
        };
    }

    pub fn get_state_store(&self) -> &StateDB {
        &self.state_store
    }

    pub fn get_base_store(&self) -> &MoveOSBaseStore {
        &self.base_store
    }
}

impl NodeStore for MoveOSStore {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>> {
        self.state_store.get(*hash)
    }

    fn put(&self, key: H256, node: Vec<u8>) -> Result<()> {
        self.state_store.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(nodes.into_iter().collect());
        self.state_store.write_batch(batch)
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

impl EventStore for MoveOSStore {
    fn save_events(
        &self,
        txn_info_id: H256,
        events: Vec<Event>,
    ) -> Result<(), Error> {
        self.event_store.save_events(txn_info_id, events)
    }

    fn get_events(
        &self,
        txn_info_id: H256,
    ) -> Result<Option<Vec<Event>>, Error> {
        self.event_store.get(txn_info_id)
    }
}

impl TransactionStore for MoveOSStore {


    fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) {
        self.transaction_store.save_tx_exec_info()
    }
    fn get_tx_exec_info(&self, tx_hash: H256) -> Option<TransactionExecutionInfo>;
    fn multi_get_tx_exec_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Vec<Option<TransactionExecutionInfo>>;


    fn get_transaction(&self, txn_hash: H256) -> Result<Option<Transaction>, Error> {
        self.transaction_store.get(txn_hash)
    }

    fn save_transaction(&self, txn: Transaction) -> Result<(), Error> {
        self.transaction_store.put(txn.id(), txn)
    }

    fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<(), Error> {
        self.transaction_store.save_transaction_batch(txn_vec)
    }

    fn get_transactions(
        &self,
        txn_hash_vec: Vec<H256>,
    ) -> Result<Vec<Option<Transaction>>, Error> {
        self.transaction_store.multiple_get(txn_hash_vec)
    }
}

/// Chain store define
pub trait Store:
    NodeStore
    + TransactionStore
    + EventStore
    + IntoSuper<dyn NodeStore>
{
    fn get_transaction_info_by_block_and_index(
        &self,
        block_id: H256,
        idx: u64,
    ) -> Result<Option<RichTransactionInfo>> {
        let txn_infos = self.get_block_txn_info_ids(block_id)?;
        match txn_infos.get(idx as usize) {
            None => Ok(None),
            Some(info_hash) => self.get_transaction_info(*info_hash),
        }
    }

    fn get_block_transaction_infos(
        &self,
        block_id: H256,
    ) -> Result<Vec<RichTransactionInfo>, Error> {
        let txn_info_ids = self.get_block_txn_info_ids(block_id)?;
        let mut txn_infos = vec![];
        let txn_opt_infos = self.get_transaction_infos(txn_info_ids.clone())?;

        for (i, info) in txn_opt_infos.into_iter().enumerate() {
            match info {
                Some(info) => txn_infos.push(info),
                None => bail!(
                    "invalid state: txn info {:?} of block {} should exist",
                    txn_info_ids.get(i),
                    block_id
                ),
            }
        }
        Ok(txn_infos)
    }

    fn get_accumulator_store(
        &self,
        accumulator_type: AccumulatorStoreType,
    ) -> Arc<dyn AccumulatorTreeStore>;
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
    fn get_accumulator_store(
        &self,
        accumulator_type: AccumulatorStoreType,
    ) -> Arc<dyn AccumulatorTreeStore> {
        match accumulator_type {
            AccumulatorStoreType::Block => Arc::new(self.block_accumulator_store.clone()),
            AccumulatorStoreType::Transaction => {
                Arc::new(self.transaction_accumulator_store.clone())
            }
        }
    }
}