// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::{ObjectEntity, RootObjectEntity};
use raw_store::rocks::RocksDB;
use raw_store::StoreInstance;
use rooch_config::RoochOpt;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;
use tracing::info;

pub mod export;
pub mod genesis_utxo;
pub mod import;

pub const BATCH_SIZE: usize = 5000;

pub const GLOBAL_STATE_TYPE_PREFIX: &str = "states";
pub const GLOBAL_STATE_TYPE_ROOT: &str = "states_root";
pub const GLOBAL_STATE_TYPE_OBJECT: &str = "states_object";
pub const GLOBAL_STATE_TYPE_FIELD: &str = "states_field";
pub fn init_statedb(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> Result<(RootObjectEntity, MoveOSStore)> {
    // Reconstruct RoochOpt
    let opt = RoochOpt::new_with_default(base_data_dir, chain_id)?;

    let store_config = opt.store_config();

    let moveos_db_path = store_config.get_moveos_store_dir();
    let moveos_store =
        MoveOSStore::new_with_instance(StoreInstance::new_db_instance(RocksDB::new(
            moveos_db_path,
            moveos_store::StoreMeta::get_column_family_names().to_vec(),
            store_config.rocksdb_config(true),
            None,
        )?))?;
    let startup_info = moveos_store.config_store.get_startup_info()?;

    if let Some(ref startup_info) = startup_info {
        info!("Load startup info {:?}", startup_info);
    }
    let root = startup_info
        .map(|s| s.into_root_object())
        .unwrap_or(ObjectEntity::genesis_root_object());

    Ok((root, moveos_store))
}
