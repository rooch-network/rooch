// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use anyhow::Result;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::ObjectMeta;
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
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
    pub fn init(config: &StoreConfig, registry: &Registry) -> Result<Self> {
        let (store_dir, indexer_dir) = (config.get_store_dir(), config.get_indexer_dir());

        let mut column_families = moveos_store::StoreMeta::get_column_family_names().to_vec();
        column_families.append(&mut rooch_store::StoreMeta::get_column_family_names().to_vec());
        //ensure no duplicate column families
        {
            let mut set = HashSet::new();
            column_families.iter().for_each(|cf| {
                if !set.insert(cf) {
                    panic!("Duplicate column family: {}", cf);
                }
            });
        }

        let db_metrics = DBMetrics::get_or_init(registry).clone();
        let instance = StoreInstance::new_db_instance(
            RocksDB::new(store_dir, column_families, config.rocksdb_config())?,
            db_metrics,
        );

        let moveos_store = MoveOSStore::new_with_instance(instance.clone(), registry)?;

        let rooch_store = RoochStore::new_with_instance(instance, registry)?;

        let indexer_store = IndexerStore::new(indexer_dir.clone(), registry)?;
        let indexer_reader = IndexerReader::new(indexer_dir, registry)?;

        Ok(Self {
            moveos_store,
            rooch_store,
            indexer_store,
            indexer_reader,
        })
    }

    pub fn init_with_mock_metrics_for_test(config: &StoreConfig) -> Result<Self> {
        let registry = prometheus::Registry::new();
        Self::init(config, &registry)
    }

    pub fn latest_root(&self) -> Result<Option<ObjectMeta>> {
        let startup_info = self.moveos_store.config_store.get_startup_info()?;

        Ok(startup_info.map(|s| s.into_root_metadata()))
    }
}
