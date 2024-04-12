// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config_store::{ConfigDBStore, ConfigStore};
use crate::event_store::{EventDBStore, EventStore};
use crate::state_store::statedb::StateDBStore;
use crate::state_store::NodeDBStore;
use crate::transaction_store::{TransactionDBStore, TransactionStore};
use anyhow::{Error, Result};
use move_core_types::language_storage::StructTag;
use moveos_config::store_config::RocksdbConfig;
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::{Event, EventID, TransactionEvent};
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::object::RootObjectEntity;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{KeyState, State};
use moveos_types::state_resolver::StateResolver;
use moveos_types::transaction::TransactionExecutionInfo;
use once_cell::sync::Lazy;
use raw_store::rocks::RocksDB;
use raw_store::{ColumnFamilyName, StoreInstance};
use smt::NodeReader;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::sync::Arc;

pub mod accumulator_store;
pub mod config_store;
pub mod event_store;
pub mod state_store;
#[cfg(test)]
mod tests;
pub mod transaction_store;

// pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
pub const STATE_NODE_PREFIX_NAME: ColumnFamilyName = "state_node";
pub const TRANSACTION_PREFIX_NAME: ColumnFamilyName = "transaction";
pub const EVENT_PREFIX_NAME: ColumnFamilyName = "event";
pub const EVENT_HANDLE_PREFIX_NAME: ColumnFamilyName = "event_handle";
pub const CONFIG_STARTUP_INFO_PREFIX_NAME: ColumnFamilyName = "config_startup_info";
pub const CONFIG_GENESIS_PREFIX_NAME: ColumnFamilyName = "config_genesis";

///db store use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
static VEC_PREFIX_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        STATE_NODE_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        EVENT_PREFIX_NAME,
        EVENT_HANDLE_PREFIX_NAME,
        CONFIG_STARTUP_INFO_PREFIX_NAME,
        CONFIG_GENESIS_PREFIX_NAME,
    ]
});

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StoreMeta {}

impl StoreMeta {
    pub fn get_column_family_names() -> &'static [ColumnFamilyName] {
        &VEC_PREFIX_NAME
    }
}

#[derive(Clone)]
pub struct MoveOSDB {
    pub node_store: NodeDBStore,
    pub event_store: EventDBStore,
    pub transaction_store: TransactionDBStore,
    pub config_store: ConfigDBStore,
}

impl MoveOSDB {
    pub fn mock_store_instance(data_dir: Option<&Path>) -> StoreInstance {
        let tmpdir = moveos_config::temp_dir();
        let db_path = data_dir.unwrap_or(tmpdir.path());
        StoreInstance::new_db_instance(
            RocksDB::new(
                db_path,
                StoreMeta::get_column_family_names().to_vec(),
                RocksdbConfig::default(),
                None,
            )
            .unwrap(),
        )
    }

    pub fn mock_moveosdb() -> Result<Self> {
        Self::new(Self::mock_store_instance(None))
    }

    pub fn mock_moveosdb_with_data_dir(data_dir: &Path) -> Result<Self> {
        Self::new(Self::mock_store_instance(Some(data_dir)))
    }

    pub fn new(instance: StoreInstance) -> Result<Self> {
        let store = Self {
            node_store: NodeDBStore::new(instance.clone()),
            event_store: EventDBStore::new(instance.clone()),
            transaction_store: TransactionDBStore::new(instance.clone()),
            config_store: ConfigDBStore::new(instance),
        };
        Ok(store)
    }
}

#[derive(Clone)]
pub struct MoveOSStore {
    pub statedb: StateDBStore,
    pub moveosdb: MoveOSDB,
}

impl MoveOSStore {
    pub fn mock_moveos_store() -> Result<Self> {
        let moveosdb = MoveOSDB::mock_moveosdb()?;
        Self::new(moveosdb)
    }

    pub fn mock_moveos_store_with_data_dir(data_dir: &Path) -> Result<Self> {
        let moveosdb = MoveOSDB::mock_moveosdb_with_data_dir(data_dir)?;
        Self::new(moveosdb)
    }

    pub fn new(moveosdb: MoveOSDB) -> Result<Self> {
        let store = Self {
            statedb: StateDBStore::new(moveosdb.node_store.clone()),
            moveosdb,
        };
        Ok(store)
    }

    pub fn new_with_root(moveosdb: MoveOSDB, root: RootObjectEntity) -> Result<Self> {
        let store = Self {
            statedb: StateDBStore::new_with_root(moveosdb.node_store.clone(), root),
            moveosdb,
        };
        Ok(store)
    }

    pub fn get_event_store(&self) -> &EventDBStore {
        &self.moveosdb.event_store
    }

    pub fn get_transaction_store(&self) -> &TransactionDBStore {
        &self.moveosdb.transaction_store
    }

    pub fn get_state_node_store(&self) -> &NodeDBStore {
        &self.moveosdb.node_store
    }

    pub fn get_config_store(&self) -> &ConfigDBStore {
        &self.moveosdb.config_store
    }

    pub fn get_state_store(&self) -> &StateDBStore {
        &self.statedb
    }

    pub fn get_state_store_mut(&mut self) -> &mut StateDBStore {
        &mut self.statedb
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

impl StateResolver for MoveOSStore {
    fn resolve_table_item(
        &self,
        handle: &ObjectID,
        key: &KeyState,
    ) -> std::result::Result<Option<State>, Error> {
        self.statedb.resolve_table_item(handle, key)
    }

    fn list_table_items(
        &self,
        handle: &ObjectID,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> std::result::Result<Vec<(KeyState, State)>, Error> {
        self.statedb.list_table_items(handle, cursor, limit)
    }

    fn root_object(&self) -> RootObjectEntity {
        self.statedb.root_object()
    }
}
