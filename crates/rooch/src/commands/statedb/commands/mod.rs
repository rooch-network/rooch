// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::OutPoint;
use chrono::{DateTime, Local};
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::ObjectState;
use redb::{ReadOnlyTable, TableDefinition};
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

const UTXO_ORD_MAP_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("utxo_ord_map");

pub fn init_genesis_job(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> (ObjectState, MoveOSStore, SystemTime) {
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

pub fn get_ord_by_outpoint(
    utxo_ord_map: Option<Arc<ReadOnlyTable<&[u8], &[u8]>>>,
    outpoint: OutPoint,
) -> Option<Vec<ObjectID>> {
    if let Some(db) = utxo_ord_map {
        let key = bcs::to_bytes(&outpoint).unwrap();
        let value = db.get(key.as_slice()).unwrap();
        value.map(|value| bcs::from_bytes(value.value()).unwrap())
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
    kvs.shrink_to_fit();
    new_len
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::Hash;
    use bitcoin::OutPoint;
    use redb::Database;
    use std::iter;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_ord_by_outpoint() {
        let obj_ids: Vec<ObjectID> = iter::repeat_with(ObjectID::random).take(3).collect();

        let db_path = NamedTempFile::new().unwrap();
        let utxo_ord_map_db = Database::create(db_path).unwrap();
        let utxo_ord_map = Arc::new(utxo_ord_map_db);

        let outpoint = OutPoint {
            txid: Hash::all_zeros(),
            vout: 1,
        };
        let write_txn = utxo_ord_map.clone().begin_write().unwrap();

        {
            let mut table = write_txn.open_table(UTXO_ORD_MAP_TABLE).unwrap();
            table
                .insert(
                    bcs::to_bytes(&outpoint).unwrap().as_slice(),
                    bcs::to_bytes(&obj_ids).unwrap().as_slice(),
                )
                .unwrap();
        }
        write_txn.commit().unwrap();

        let read_txn = utxo_ord_map.begin_read().unwrap();
        let read_table = read_txn.open_table(UTXO_ORD_MAP_TABLE).unwrap();
        assert_eq!(
            None,
            get_ord_by_outpoint(
                Some(Arc::new(read_table)),
                OutPoint {
                    txid: Hash::all_zeros(),
                    vout: 0,
                }
            )
        );

        let read_txn = utxo_ord_map.begin_read().unwrap();
        let read_table = read_txn.open_table(UTXO_ORD_MAP_TABLE).unwrap();
        assert_eq!(
            obj_ids.clone(),
            get_ord_by_outpoint(Some(Arc::new(read_table)), outpoint).unwrap()
        );
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
