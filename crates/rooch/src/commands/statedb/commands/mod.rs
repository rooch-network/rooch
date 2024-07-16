// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::{OutPoint, PublicKey, ScriptBuf};
use chrono::{DateTime, Local};
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::{ObjectID, ObjectMeta};
use redb::{ReadOnlyTable, TableDefinition};
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::address::BitcoinAddress;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::rooch_network::RoochChainID;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
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

pub const SCRIPT_TYPE_P2MS: &str = "p2ms";
pub const SCRIPT_TYPE_P2PK: &str = "p2pk";
pub const SCRIPT_TYPE_NON_STANDARD: &str = "non-standard";

pub fn init_genesis_job(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> (ObjectMeta, MoveOSStore, SystemTime) {
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

// drive BitcoinAddress from data source
pub fn drive_bitcoin_address(
    origin_address: String,
    script: String,
    script_type: String,
) -> Option<BitcoinAddress> {
    if !origin_address.is_empty() {
        return Some(BitcoinAddress::from_str(origin_address.as_str()).unwrap());
    }
    if SCRIPT_TYPE_NON_STANDARD.eq(script_type.as_str()) {
        return None;
    }
    // Try to derive address from script
    let script_buf = ScriptBuf::from_hex(script.as_str()).unwrap();
    let bitcoin_address: BitcoinAddress = BitcoinAddress::from(&script_buf);
    if bitcoin_address != BitcoinAddress::default() {
        return Some(bitcoin_address);
    }
    // Try to derive address from p2pk pubkey
    if SCRIPT_TYPE_P2PK.eq(script_type.as_str()) {
        let pubkey = match PublicKey::from_str(script.as_str()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return None;
            }
        };
        let pubkey_hash = pubkey.pubkey_hash();
        return Some(BitcoinAddress::new_p2pkh(&pubkey_hash));
    };
    return None;
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

    #[test]
    fn test_drive_bitcoin_address() {
        // non-empty address
        let bitcoin_address = drive_bitcoin_address(
            "bc1qsn7v0rwezflwd6pk7xxf25zhjw9wkvmympm7tk".to_string(),
            "".to_string(),
            SCRIPT_TYPE_NON_STANDARD.to_string(), // no matter what script type
        );
        assert_eq!(
            "bc1qsn7v0rwezflwd6pk7xxf25zhjw9wkvmympm7tk",
            bitcoin_address.unwrap().to_string()
        );
        // non-standard address
        let bitcoin_address = drive_bitcoin_address(
            "".to_string(),
            "".to_string(),
            SCRIPT_TYPE_NON_STANDARD.to_string(),
        );
        assert_eq!(None, bitcoin_address);
        // p2pk script
        let bitcoin_address = drive_bitcoin_address(
            "".to_string(),
            "41049434a2dd7c5b82df88f578f8d7fd14e8d36513aaa9c003eb5bd6cb56065e44b7e0227139e8a8e68e7de0a4ed32b8c90edc9673b8a7ea541b52f2a22196f7b8cfac".to_string(),
            SCRIPT_TYPE_P2PK.to_string(),
        );
        assert_eq!(
            "14vrCdzPtnHaXtDNLH4xNhceS7GV4GMw76",
            bitcoin_address.unwrap().to_string()
        );
        // p2pk pubkey
        let bitcoin_address = drive_bitcoin_address(
            "".to_string(),
            "04f254e36949ec1a7f6e9548f16d4788fb321f429b2c7d2eb44480b2ed0195cbf0c3875c767fe8abb2df6827c21392ea5cc934240b9ac46c6a56d2bd13dd0b17a9".to_string(),
            SCRIPT_TYPE_P2PK.to_string(),
        );
        let pubkey = PublicKey::from_str(
            "04f254e36949ec1a7f6e9548f16d4788fb321f429b2c7d2eb44480b2ed0195cbf0c3875c767fe8abb2df6827c21392ea5cc934240b9ac46c6a56d2bd13dd0b17a9",
        )
        .unwrap();
        assert_eq!(
            BitcoinAddress::new_p2pkh(&pubkey.pubkey_hash()),
            bitcoin_address.unwrap()
        );
        // invalid p2pk pubkey
        let bitcoin_address = drive_bitcoin_address(
            "".to_string(),
            "036c6565662c206f6e7464656b2c2067656e6965742e2e2e202020202020202020".to_string(),
            SCRIPT_TYPE_P2PK.to_string(),
        );
        assert_eq!(None, bitcoin_address);
        // invalid p2pk script
        let bitcoin_address = drive_bitcoin_address(
            "".to_string(),
            "21036c6565662c206f6e7464656b2c2067656e6965742e2e2e202020202020202020ac".to_string(),
            SCRIPT_TYPE_P2PK.to_string(),
        );
        assert_eq!(None, bitcoin_address);
        // special p2ms case: https://ordinals.com/inscription/72552729(
        // output: a353a7943a2b38318bf458b6af878b8384f48a6d10aad5b827d0550980abe3f0:0
        // script: 0014f29f9316f0f1e48116958216a8babd353b491dae
        // address: bc1q720ex9hs78jgz954sgt23w4ax5a5j8dwjj5kkm
        // )
        let script = "0014f29f9316f0f1e48116958216a8babd353b491dae";
        let bitcoin_address = drive_bitcoin_address(
            "".to_string(),
            script.to_string(),
            SCRIPT_TYPE_P2MS.to_string(),
        );
        assert_eq!(
            "bc1q720ex9hs78jgz954sgt23w4ax5a5j8dwjj5kkm",
            bitcoin_address.unwrap().to_string()
        );
        // normal p2ms (cannot get payload)
        let script = "512102047da7156b82baaed491787e77a0d94cbc00ebdbd993639382b8a41d2f8d42dd2107000000000000000000000000000000000000000000000000000000000000000052ae";
        let bitcoin_address = drive_bitcoin_address(
            "".to_string(),
            script.to_string(),
            SCRIPT_TYPE_P2MS.to_string(),
        );
        assert_eq!(None, bitcoin_address);
    }
}
