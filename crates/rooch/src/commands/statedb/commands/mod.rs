// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Local};
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::RootObjectEntity;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;
use std::time::SystemTime;

pub mod export;
pub mod genesis_ord;
pub mod genesis_utxo;
pub mod import;

pub const BATCH_SIZE: usize = 5000;

pub const GLOBAL_STATE_TYPE_PREFIX: &str = "states";
pub const GLOBAL_STATE_TYPE_ROOT: &str = "states_root";
pub const GLOBAL_STATE_TYPE_OBJECT: &str = "states_object";
pub const GLOBAL_STATE_TYPE_FIELD: &str = "states_field";

const UTXO_SEAL_INSCRIPTION_PROTOCOL: &str =
    "0000000000000000000000000000000000000000000000000000000000000004::ord::Inscription";

pub fn init_genesis_job(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> (RootObjectEntity, MoveOSStore, SystemTime) {
    let start_time = SystemTime::now();
    let datetime: DateTime<Local> = start_time.into();

    let opt = RoochOpt::new_with_default(base_data_dir.clone(), chain_id.clone(), None).unwrap();
    let rooch_db = RoochDB::init(opt.store_config()).unwrap();
    let root = rooch_db.latest_root().unwrap().unwrap();

    let utxo_store_id = BitcoinUTXOStore::object_id();
    let address_mapping_id = RoochToBitcoinAddressMapping::object_id();
    let inscription_store_id = InscriptionStore::object_id();

    println!("task progress started at {}", datetime,);
    println!("root object: {:?}", root);
    println!("utxo_store_id: {:?}", utxo_store_id);
    println!(
        "rooch to bitcoin address_mapping_id: {:?}",
        address_mapping_id
    );
    println!("inscription_store_id: {:?}", inscription_store_id);
    (root, rooch_db.moveos_store, start_time)
}
