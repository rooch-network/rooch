// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::RootObjectEntity;
use raw_store::{rocks::RocksDB, StoreInstance};
use rooch_config::store_config::StoreConfig;
use rooch_indexer::{indexer_reader::IndexerReader, IndexerStore};
use rooch_store::RoochStore;

#[derive(Clone)]
pub struct RoochDB {
    pub moveos_store: MoveOSStore,
    pub rooch_store: RoochStore,
    pub indexer_store: IndexerStore,
    pub indexer_reader: IndexerReader,
}

impl RoochDB {
    pub fn init(config: &StoreConfig) -> Result<Self> {
        let (rooch_store_dir, moveos_store_dir, indexer_store_dir) = (
            config.get_rooch_store_dir(),
            config.get_moveos_store_dir(),
            config.get_indexer_store_dir(),
        );

        //TODO should we merge the moveos_store with rooch_store use one StoreInstance.
        let moveos_store =
            MoveOSStore::new_with_instance(StoreInstance::new_db_instance(RocksDB::new(
                moveos_store_dir,
                moveos_store::StoreMeta::get_column_family_names().to_vec(),
                config.rocksdb_config(true),
                None,
            )?))?;

        let rooch_store =
            RoochStore::new_with_instance(StoreInstance::new_db_instance(RocksDB::new(
                rooch_store_dir,
                rooch_store::StoreMeta::get_column_family_names().to_vec(),
                config.rocksdb_config(false),
                None,
            )?))?;

        let indexer_store = IndexerStore::new(indexer_store_dir.clone())?;
        let indexer_reader = IndexerReader::new(indexer_store_dir)?;

        Ok(Self {
            moveos_store,
            rooch_store,
            indexer_store,
            indexer_reader,
        })
    }

    pub fn latest_root(&self) -> Result<Option<RootObjectEntity>> {
        let startup_info = self.moveos_store.config_store.get_startup_info()?;

        Ok(startup_info.map(|s| s.into_root_object()))
    }
}
