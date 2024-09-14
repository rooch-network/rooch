// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::RegistryService;
use moveos_types::moveos_std::object::ObjectMeta;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;
use std::time::SystemTime;

pub mod revert;
pub mod rollback;

fn init(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> (ObjectMeta, RoochDB, SystemTime) {
    let start_time = SystemTime::now();

    let opt = RoochOpt::new_with_default(base_data_dir, chain_id, None).unwrap();
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();
    let root = rooch_db.latest_root().unwrap().unwrap();
    (root, rooch_db, start_time)
}
