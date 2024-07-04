// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::OutPoint;
use chrono::{DateTime, Local};
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::{ObjectID, RootObjectEntity};
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::rooch_network::RoochChainID;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
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

pub fn concatenate_object_id_merge(
    _key: &[u8],              // the key being merged
    old_value: Option<&[u8]>, // the previous value, if one existed
    merged_bytes: &[u8],      // the new bytes(object_id) being merged in
) -> Option<Vec<u8>> {
    // set the new value, return None to delete
    let mut object_ids: Vec<ObjectID> = old_value
        .map(|ov| bcs::from_bytes(ov).unwrap())
        .unwrap_or_default();

    let new_object_id = bcs::from_bytes(merged_bytes).unwrap();
    object_ids.push(new_object_id);

    Some(bcs::to_bytes(&object_ids).unwrap())
}

pub fn insert_ord_to_output(
    utxo_ord_map: Arc<sled::Db>,
    outpoint: OutPoint,
    obj_id_bytes: Vec<u8>,
) {
    let key = bcs::to_bytes(&outpoint).unwrap();
    utxo_ord_map.merge(key, obj_id_bytes).unwrap();
}

pub fn get_ord_by_outpoint(
    utxo_ord_map: Option<Arc<sled::Db>>,
    outpoint: OutPoint,
) -> Option<Vec<ObjectID>> {
    if let Some(db) = utxo_ord_map {
        let key = bcs::to_bytes(&outpoint).unwrap();
        let value = db.get(key).unwrap();
        value.map(|value| bcs::from_bytes(&value).unwrap())
    } else {
        None
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UTXOOrds {
    pub utxo: OutPoint,
    pub ords: Vec<ObjectID>,
}

pub fn sort_merge_utxo_ords(kvs: &mut Vec<UTXOOrds>) -> usize {
    if kvs.is_empty() {
        return 0;
    }

    // Step 1: Sort by utxo
    kvs.sort_by(|a, b| a.utxo.cmp(&b.utxo));
    // Step 2: Merge in place
    let mut write_index = 0;
    for read_index in 1..kvs.len() {
        if kvs[write_index].utxo == kvs[read_index].utxo {
            let drained_ords: Vec<ObjectID> = kvs[read_index].ords.drain(..).collect();
            kvs[write_index].ords.extend(drained_ords);
        } else {
            write_index += 1;
            if write_index != read_index {
                kvs[write_index] = std::mem::take(&mut kvs[read_index]);
            }
        }
    }

    // Truncate the vector to remove the unused elements
    let new_len = write_index + 1;
    kvs.truncate(new_len);
    new_len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::statedb::commands::get_ord_by_outpoint;
    use bitcoin::hashes::Hash;
    use bitcoin::OutPoint;
    use std::iter;

    #[test]
    fn test_concatenate_object_id_merge() {
        let db_config = sled::Config::new().temporary(true);
        let db = db_config.open().unwrap();
        db.set_merge_operator(concatenate_object_id_merge);
        let utxo_ord_map = Arc::new(db);
        let outpoint = OutPoint::default();

        let obj_ids = get_ord_by_outpoint(Some(utxo_ord_map.clone()), outpoint);
        assert_eq!(None, obj_ids);

        let obj_id_0 = ObjectID::random();
        let value = bcs::to_bytes(&obj_id_0).unwrap();
        insert_ord_to_output(utxo_ord_map.clone(), outpoint, value);
        let obj_ids = get_ord_by_outpoint(Some(utxo_ord_map.clone()), outpoint).unwrap();
        assert_eq!(obj_ids[0], obj_id_0);

        let obj_id_1 = ObjectID::random();
        let value = bcs::to_bytes(&obj_id_1).unwrap();
        insert_ord_to_output(utxo_ord_map.clone(), outpoint, value);
        let obj_ids = get_ord_by_outpoint(Some(utxo_ord_map.clone()), outpoint).unwrap();
        assert_eq!(obj_ids, vec![obj_id_0, obj_id_1]);
    }

    #[test]
    fn test_sort_merge_utxo_ords() {
        let obj_ids: Vec<ObjectID> = iter::repeat_with(ObjectID::random).take(3).collect();

        let mut kvs = vec![
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 0,
                },
                ords: vec![obj_ids[0].clone()],
            },
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 1,
                },
                ords: vec![obj_ids[1].clone()],
            },
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 0,
                },
                ords: vec![obj_ids[2].clone()],
            },
        ];

        let new_len = sort_merge_utxo_ords(&mut kvs);

        assert_eq!(new_len, 2);
        assert_eq!(
            kvs,
            vec![
                UTXOOrds {
                    utxo: OutPoint {
                        txid: Hash::all_zeros(),
                        vout: 0,
                    },
                    ords: vec![obj_ids[0].clone(), obj_ids[2].clone()],
                },
                UTXOOrds {
                    utxo: OutPoint {
                        txid: Hash::all_zeros(),
                        vout: 1,
                    },
                    ords: vec![obj_ids[1].clone()],
                },
            ]
        );
    }

    #[test]
    fn test_sort_merge_utxo_ords_empty() {
        let mut kvs: Vec<UTXOOrds> = Vec::new();

        let new_len = sort_merge_utxo_ords(&mut kvs);

        assert_eq!(new_len, 0);
        assert!(kvs.is_empty());
    }

    #[test]
    fn test_sort_merge_utxo_ords_no_merge_needed() {
        let obj_ids: Vec<ObjectID> = iter::repeat_with(ObjectID::random).take(3).collect();

        let mut kvs = vec![
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 0,
                },
                ords: vec![obj_ids[0].clone()],
            },
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 1,
                },
                ords: vec![obj_ids[1].clone()],
            },
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 2,
                },
                ords: vec![obj_ids[2].clone()],
            },
        ];

        let new_len = sort_merge_utxo_ords(&mut kvs);

        assert_eq!(new_len, 3);
        assert_eq!(
            kvs,
            vec![
                UTXOOrds {
                    utxo: OutPoint {
                        txid: Hash::all_zeros(),
                        vout: 0,
                    },
                    ords: vec![obj_ids[0].clone()],
                },
                UTXOOrds {
                    utxo: OutPoint {
                        txid: Hash::all_zeros(),
                        vout: 1,
                    },
                    ords: vec![obj_ids[1].clone()],
                },
                UTXOOrds {
                    utxo: OutPoint {
                        txid: Hash::all_zeros(),
                        vout: 2,
                    },
                    ords: vec![obj_ids[2].clone()],
                },
            ]
        );
    }

    #[test]
    fn test_sort_merge_utxo_ords_all_merge() {
        let obj_ids: Vec<ObjectID> = iter::repeat_with(ObjectID::random).take(3).collect();

        let mut kvs = vec![
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 0,
                },
                ords: vec![obj_ids[0].clone()],
            },
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 0,
                },
                ords: vec![obj_ids[1].clone()],
            },
            UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 0,
                },
                ords: vec![obj_ids[2].clone()],
            },
        ];

        let new_len = sort_merge_utxo_ords(&mut kvs);

        assert_eq!(new_len, 1);
        assert_eq!(
            kvs,
            vec![UTXOOrds {
                utxo: OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 0,
                },
                ords: vec![obj_ids[0].clone(), obj_ids[1].clone(), obj_ids[2].clone()],
            },]
        );
    }
}
