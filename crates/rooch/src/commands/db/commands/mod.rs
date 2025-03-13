// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use raw_store::rocks::RocksDB;
use rooch_config::RoochOpt;
use rooch_types::rooch_network::RoochChainID;
use std::collections::HashSet;
use std::path::PathBuf;

pub mod best_rollback;
pub mod changeset;
pub mod cp_cf;
pub mod drop;
pub mod get_changeset_by_order;
pub mod get_execution_info_by_hash;
pub mod get_tx_by_order;
pub mod list_anomaly;
pub mod repair;
pub mod revert;
pub mod rollback;
pub mod stat_changeset;
pub mod verify_order;

fn open_rocks(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> anyhow::Result<RocksDB> {
    let opt = RoochOpt::new_with_default(base_data_dir, chain_id, None).unwrap();
    let store_config = opt.store_config();
    let store_dir = store_config.get_store_dir();
    let mut column_families = moveos_store::StoreMeta::get_column_family_names().to_vec();
    column_families.append(&mut rooch_store::StoreMeta::get_column_family_names().to_vec());
    //ensure no duplicate column families
    {
        let mut set = HashSet::with_capacity(column_families.len());
        column_families.iter().for_each(|cf| {
            if !set.insert(cf) {
                panic!("Duplicate column family: {}", cf);
            }
        });
    }

    RocksDB::new(store_dir, column_families, store_config.rocksdb_config())
}
