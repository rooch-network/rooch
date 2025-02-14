// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config_store::{ConfigDBStore, ConfigStore, STARTUP_INFO_KEY};
use crate::event_store::{EventDBStore, EventStore};
use crate::state_store::statedb::StateDBStore;
use crate::state_store::{nodes_to_write_batch, NodeDBStore};
use crate::transaction_store::{TransactionDBStore, TransactionStore};
use accumulator::inmemory::InMemoryAccumulator;
use anyhow::{Error, Result};
use bcs::to_bytes;
use move_core_types::language_storage::StructTag;
use moveos_config::store_config::{MoveOSStoreConfig, RocksdbConfig};
use moveos_config::DataDirPath;
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::{Event, EventID, TransactionEvent};
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::onchain_features::FeatureStore;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{FieldKey, ObjectState};
use moveos_types::state_resolver::{StateKV, StateResolver, StatelessResolver};
use moveos_types::transaction::{
    RawTransactionOutput, TransactionExecutionInfo, TransactionOutput,
};
use once_cell::sync::Lazy;
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
use raw_store::rocks::batch::{WriteBatch, WriteBatchCF};
use raw_store::rocks::RocksDB;
use raw_store::traits::DBStore;
use raw_store::{ColumnFamilyName, SchemaStore, StoreInstance, WriteOp};
use smt::NodeReader;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::sync::Arc;

pub mod config_store;
pub mod event_store;
pub mod state_store;
#[cfg(test)]
mod tests;
pub mod transaction_store;

// pub const DEFAULT_COLUMN_FAMILY_NAME: ColumnFamilyName = "default";
pub const STATE_NODE_COLUMN_FAMILY_NAME: ColumnFamilyName = "state_node";
pub const TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME: ColumnFamilyName =
    "transaction_execution_info";
pub const EVENT_COLUMN_FAMILY_NAME: ColumnFamilyName = "event";
pub const EVENT_HANDLE_COLUMN_FAMILY_NAME: ColumnFamilyName = "event_handle";
pub const CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME: ColumnFamilyName = "config_startup_info";
pub const CONFIG_GENESIS_COLUMN_FAMILY_NAME: ColumnFamilyName = "config_genesis";

/// db store use cf_name vec to init
/// Please note that adding a column family needs to be added in vec simultaneously, remember！！
static VEC_COLUMN_FAMILY_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        STATE_NODE_COLUMN_FAMILY_NAME,
        TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME,
        EVENT_COLUMN_FAMILY_NAME,
        EVENT_HANDLE_COLUMN_FAMILY_NAME,
        CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME,
        CONFIG_GENESIS_COLUMN_FAMILY_NAME,
    ]
});

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StoreMeta {}

impl StoreMeta {
    pub fn get_column_family_names() -> &'static [ColumnFamilyName] {
        &VEC_COLUMN_FAMILY_NAME
    }
}

#[derive(Clone)]
pub struct MoveOSStore {
    pub node_store: NodeDBStore,
    pub event_store: EventDBStore,
    pub transaction_store: TransactionDBStore,
    pub config_store: ConfigDBStore,
    pub state_store: StateDBStore,
}

impl MoveOSStore {
    pub fn new(db_path: &Path, registry: &Registry) -> Result<Self> {
        let db_metrics = DBMetrics::get_or_init(registry).clone();
        let instance = StoreInstance::new_db_instance(
            RocksDB::new(
                db_path,
                StoreMeta::get_column_family_names().to_vec(),
                RocksdbConfig::default(),
            )?,
            db_metrics,
        );
        Self::new_with_instance(instance, registry)
    }

    pub fn new_with_instance(instance: StoreInstance, registry: &Registry) -> Result<Self> {
        let store_config = MoveOSStoreConfig::default();
        let node_store = NodeDBStore::new(instance.clone());
        let state_store =
            StateDBStore::new(node_store.clone(), registry, store_config.state_cache_size);

        let store = Self {
            node_store,
            event_store: EventDBStore::new(instance.clone()),
            transaction_store: TransactionDBStore::new(instance.clone()),
            config_store: ConfigDBStore::new(instance),
            state_store,
        };
        Ok(store)
    }

    pub fn mock_moveos_store() -> Result<(Self, DataDirPath)> {
        let tmpdir = moveos_config::temp_dir();
        let registry = prometheus::Registry::new();

        //The testcases should hold the tmpdir to prevent the tmpdir from being deleted.
        Ok((Self::new(tmpdir.path(), &registry)?, tmpdir))
    }

    pub fn get_event_store(&self) -> &EventDBStore {
        &self.event_store
    }

    pub fn get_transaction_store(&self) -> &TransactionDBStore {
        &self.transaction_store
    }

    pub fn get_state_node_store(&self) -> &NodeDBStore {
        &self.node_store
    }

    pub fn get_config_store(&self) -> &ConfigDBStore {
        &self.config_store
    }

    pub fn get_state_store(&self) -> &StateDBStore {
        &self.state_store
    }

    pub fn handle_tx_output(
        &self,
        tx_hash: H256,
        output: RawTransactionOutput,
    ) -> Result<(TransactionOutput, TransactionExecutionInfo)> {
        let RawTransactionOutput {
            status,
            mut changeset,
            events: tx_events,
            gas_used,
            is_upgrade,
            is_gas_upgrade: _,
        } = output;

        // node_store updates
        let changed_nodes = self.state_store.change_set_to_nodes(&mut changeset)?;
        // transaction_store updates
        let new_state_root = changeset.state_root;
        let size = changeset.global_size;
        let event_ids = self.event_store.save_events(tx_events.clone())?;
        let events = tx_events
            .clone()
            .into_iter()
            .zip(event_ids)
            .map(|(event, event_id)| Event::new_with_event_id(event_id, event))
            .collect::<Vec<_>>();
        let event_hashes: Vec<_> = events.iter().map(|e| e.hash()).collect();
        let event_root = InMemoryAccumulator::from_leaves(event_hashes.as_slice()).root_hash();
        let execution_info = TransactionExecutionInfo::new(
            tx_hash,
            new_state_root,
            size,
            event_root,
            gas_used,
            status.clone(),
        );
        // config_store updates
        let new_startup_info = StartupInfo::new(new_state_root, size);

        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!(
                "handle_tx_output: tx_hash: {:?}, state_root: {}, size: {}, gas_used: {}, status: {:?}",
                tx_hash,
                new_state_root,
                size,
                gas_used,
                status
            );
        }

        // atomic save updates
        let inner_store = self.node_store.get_store().store();
        let mut cf_batches: Vec<WriteBatchCF> = Vec::new();
        let write_batch = nodes_to_write_batch(changed_nodes);
        cf_batches.push(WriteBatchCF {
            batch: write_batch,
            cf_name: STATE_NODE_COLUMN_FAMILY_NAME.to_string(),
        });
        cf_batches.push(WriteBatchCF {
            batch: WriteBatch::new_with_rows(vec![(
                to_bytes(STARTUP_INFO_KEY).unwrap(),
                WriteOp::Value(to_bytes(&new_startup_info).unwrap()),
            )]),
            cf_name: CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME.to_string(),
        });
        cf_batches.push(WriteBatchCF {
            batch: WriteBatch::new_with_rows(vec![(
                to_bytes(&tx_hash).unwrap(),
                WriteOp::Value(to_bytes(&execution_info).unwrap()),
            )]),
            cf_name: TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME.to_string(),
        });
        // use non-sync write here:
        // 1. we could replay tx from rooch store(which has sync write after sequenced) at startup.
        // 2. output write sequentially
        inner_store.write_cf_batch(cf_batches, false)?;

        let out = TransactionOutput::new(status, changeset, events, gas_used, is_upgrade);

        Ok((out, execution_info))
    }
}

impl Display for MoveOSStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "statedb")?;
        write!(f, "event_store")?;
        write!(f, "transaction_store")?;
        write!(f, "node_store")?;
        Ok(())
    }
}

impl Debug for MoveOSStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl NodeReader for MoveOSStore {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>> {
        self.get_state_node_store().get(hash)
    }
}

impl EventStore for MoveOSStore {
    fn save_events(&self, events: Vec<TransactionEvent>) -> Result<Vec<EventID>> {
        self.get_event_store().save_events(events)
    }

    fn get_event(&self, event_id: EventID) -> Result<Option<Event>> {
        self.get_event_store().get_event(event_id)
    }

    fn multi_get_events(&self, event_ids: Vec<EventID>) -> Result<Vec<Option<Event>>> {
        self.get_event_store().multi_get_events(event_ids)
    }

    fn get_events_by_event_handle_id(
        &self,
        event_handle_id: &ObjectID,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<Event>> {
        self.get_event_store().get_events_by_event_handle_id(
            event_handle_id,
            cursor,
            limit,
            descending_order,
        )
    }

    fn get_events_by_event_handle_type(
        &self,
        event_handle_type: &StructTag,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<Event>> {
        self.get_event_store().get_events_by_event_handle_type(
            event_handle_type,
            cursor,
            limit,
            descending_order,
        )
    }
}

impl TransactionStore for MoveOSStore {
    fn save_tx_execution_info(&self, tx_execution_info: TransactionExecutionInfo) -> Result<()> {
        self.get_transaction_store()
            .save_tx_execution_info(tx_execution_info)
    }

    fn get_tx_execution_info(&self, tx_hash: H256) -> Result<Option<TransactionExecutionInfo>> {
        self.get_transaction_store().get_tx_execution_info(tx_hash)
    }

    fn multi_get_tx_execution_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        self.get_transaction_store()
            .multi_get_tx_execution_infos(tx_hashes)
    }

    fn remove_tx_execution_info(&self, tx_hash: H256) -> Result<()> {
        self.get_transaction_store()
            .remove_tx_execution_info(tx_hash)
    }
}

impl ConfigStore for MoveOSStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.get_config_store().get_startup_info()
    }

    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.get_config_store().save_startup_info(startup_info)
    }

    fn get_genesis(&self) -> Result<Option<GenesisInfo>> {
        self.get_config_store().get_genesis()
    }

    fn save_genesis(&self, genesis_info: GenesisInfo) -> Result<()> {
        self.get_config_store().save_genesis(genesis_info)
    }
}

/// Moveos store define
pub trait Store:
    NodeReader + TransactionStore + EventStore + ConfigStore + IntoSuper<dyn NodeReader>
{
}

pub trait IntoSuper<Super: ?Sized> {
    fn as_super(&self) -> &Super;
    fn as_super_mut(&mut self) -> &mut Super;
    fn into_super(self: Box<Self>) -> Box<Super>;
    fn into_super_arc(self: Arc<Self>) -> Arc<Super>;
}

impl<'a, T: 'a + NodeReader> IntoSuper<dyn NodeReader + 'a> for T {
    fn as_super(&self) -> &(dyn NodeReader + 'a) {
        self
    }
    fn as_super_mut(&mut self) -> &mut (dyn NodeReader + 'a) {
        self
    }
    fn into_super(self: Box<Self>) -> Box<dyn NodeReader + 'a> {
        self
    }
    fn into_super_arc(self: Arc<Self>) -> Arc<dyn NodeReader + 'a> {
        self
    }
}

impl Store for MoveOSStore {}

impl StatelessResolver for MoveOSStore {
    fn get_field_at(&self, state_root: H256, key: &FieldKey) -> Result<Option<ObjectState>, Error> {
        self.get_state_store().get_field_at(state_root, key)
    }

    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>, Error> {
        self.get_state_store()
            .list_fields_at(state_root, cursor, limit)
    }
}

pub fn load_feature_store_object<Resolver: StateResolver>(
    state_resolver: &Resolver,
) -> Option<FeatureStore> {
    let feature_store_object = state_resolver
        .get_object(&FeatureStore::feature_store_object_id())
        .unwrap_or(None);

    match feature_store_object {
        None => None,
        Some(future_store_state) => {
            let future_store_result = future_store_state.into_object::<FeatureStore>();
            match future_store_result {
                Ok(future_store) => Some(future_store.value),
                Err(_) => None,
            }
        }
    }
}
