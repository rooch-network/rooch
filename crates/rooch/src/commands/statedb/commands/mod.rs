// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_store::{MoveOSDB, MoveOSStore};
use moveos_types::moveos_std::object::{ObjectEntity, RootObjectEntity};
use raw_store::rocks::RocksDB;
use raw_store::StoreInstance;
use rooch_config::store_config::StoreConfig;
use rooch_config::{BaseConfig, RoochOpt};
use rooch_types::chain_id::RoochChainID;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

pub mod export;
pub mod genesis_utxo;
pub mod import;

pub fn init_statedb(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> Result<(RootObjectEntity, MoveOSStore)> {
    // Reconstruct RoochOpt
    let opt = RoochOpt::new_with_default(base_data_dir, chain_id);

    //Init store
    let base_config = BaseConfig::load_with_opt(&opt)?;
    let arc_base_config = Arc::new(base_config);
    let mut store_config = StoreConfig::default();
    store_config.merge_with_opt_with_init(&opt, Arc::clone(&arc_base_config), false)?;

    let moveos_db_path = store_config.get_moveos_store_dir();
    let moveosdb = MoveOSDB::new(StoreInstance::new_db_instance(RocksDB::new(
        moveos_db_path,
        moveos_store::StoreMeta::get_column_family_names().to_vec(),
        store_config.rocksdb_config(true),
        None,
    )?))?;
    let startup_info = moveosdb.config_store.get_startup_info()?;

    if let Some(ref startup_info) = startup_info {
        info!("Load startup info {:?}", startup_info);
    }
    let root = startup_info
        .map(|s| s.into_root_object())
        .unwrap_or(ObjectEntity::genesis_root_object());

    let moveos_store = MoveOSStore::new(moveosdb)?;

    Ok((root, moveos_store))
}
