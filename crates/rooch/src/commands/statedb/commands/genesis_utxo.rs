// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::import::{apply_fields, apply_nodes};
use crate::commands::statedb::commands::{
    get_ord_by_outpoint, UTXO_ORD_MAP_TABLE, UTXO_SEAL_INSCRIPTION_PROTOCOL,
};
use anyhow::{Error, Result};
use bitcoin::{OutPoint, PublicKey, Txid};
use chrono::{DateTime, Local};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::{
    ObjectEntity, ObjectID, GENESIS_STATE_ROOT, SHARED_OBJECT_FLAG_MASK, SYSTEM_OWNER_ADDRESS,
};
use moveos_types::moveos_std::simple_multimap::{Element, SimpleMultiMap};
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{FieldKey, ObjectState};
use redb::{Database, ReadOnlyTable};
use rooch_common::fs::file_cache::FileCacheManager;
use rooch_config::{RoochOpt, R_OPT_NET_HELP};
use rooch_db::RoochDB;
use rooch_types::address::BitcoinAddress;
use rooch_types::addresses::BITCOIN_MOVE_ADDRESS;
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::bitcoin::{types, utxo};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::into_address::IntoAddress;
use rooch_types::rooch_network::RoochChainID;
use serde::{Deserialize, Serialize};
use smt::UpdateSet;
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::SystemTime;

pub const SCRIPT_TYPE_P2MS: &str = "p2ms";
pub const SCRIPT_TYPE_P2PK: &str = "p2pk";
pub const SCRIPT_TYPE_NON_STANDARD: &str = "non-standard";

/// Genesis Import UTXO
#[derive(Debug, Parser)]
pub struct GenesisUTXOCommand {
    // #[clap(long, short = 'i', parse(from_os_str))]
    #[clap(long, short = 'i')]
    /// import input file. like ~/.rooch/local/utxo.csv or utxo.csv
    /// The file format is csv, and the first line is the header, the header is as follows:
    /// count,txid,vout,height,coinbase,amount,script,type,address
    pub input: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long, short = 'b', default_value = "2097152")]
    pub batch_size: Option<usize>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl GenesisUTXOCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let input_path = self.input.clone();
        let batch_size = self.batch_size.unwrap();
        let (root, moveos_store, start_time) = self.init();
        let pre_root_state_root = root.state_root();
        let (tx, rx) = mpsc::sync_channel(2);
        let moveos_store = Arc::new(moveos_store);
        let produce_updates_thread =
            thread::spawn(move || produce_utxo_updates(tx, input_path, batch_size, None));
        let apply_updates_thread = thread::spawn(move || {
            apply_utxo_updates_to_state(
                rx,
                moveos_store,
                root.size(),
                pre_root_state_root,
                None,
                start_time,
            );
        });
        produce_updates_thread.join().unwrap();
        apply_updates_thread.join().unwrap();

        Ok(())
    }

    fn init(self) -> (ObjectState, MoveOSStore, SystemTime) {
        let start_time = SystemTime::now();
        let datetime: DateTime<Local> = start_time.into();

        let opt = RoochOpt::new_with_default(self.base_data_dir, self.chain_id, None).unwrap();
        let rooch_db = RoochDB::init(opt.store_config()).unwrap();
        let root = rooch_db.latest_root().unwrap().unwrap();

        let utxo_store_id = BitcoinUTXOStore::object_id();
        let address_mapping_id = RoochToBitcoinAddressMapping::object_id();

        println!(
            "task progress started at {}, batch_size: {}",
            datetime,
            self.batch_size.unwrap()
        );
        println!("root object: {:?}", root);
        println!("utxo_store_id: {:?}", utxo_store_id);
        println!(
            "rooch to bitcoin address_mapping_id: {:?}",
            address_mapping_id
        );
        (root, rooch_db.moveos_store, start_time)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UTXOData {
    /// The txid of the UTXO
    pub txid: String,
    /// The vout of the UTXO
    pub vout: u32,
    pub value: u64,
    pub script: String,
    pub script_type: String,
    pub address: String,
}

impl UTXOData {
    pub fn new(
        txid: String,
        vout: u32,
        value: u64,
        script: String,
        script_type: String,
        address: String,
    ) -> Self {
        Self {
            txid,
            vout,
            value,
            script,
            script_type,
            address,
        }
    }

    pub fn is_valid_empty_address(&self) -> bool {
        SCRIPT_TYPE_P2PK.eq(self.script_type.as_str())
            || SCRIPT_TYPE_P2MS.eq(self.script_type.as_str())
            || SCRIPT_TYPE_NON_STANDARD.eq(self.script_type.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressMappingData {
    pub origin_address: String,
    pub baddress: BitcoinAddress,
    pub address: AccountAddress,
}

impl AddressMappingData {
    pub fn new(origin_address: String, baddress: BitcoinAddress, address: AccountAddress) -> Self {
        Self {
            origin_address,
            baddress,
            address,
        }
    }

    pub fn into_state(self) -> ObjectState {
        let parent_id = RoochToBitcoinAddressMapping::object_id();
        //Rooch address to bitcoin address dynamic field: name is rooch address, value is bitcoin address
        ObjectEntity::new_dynamic_field(parent_id, self.address, self.baddress).into_state()
    }
}

// csv format: count,txid,vout,height,coinbase,amount,script,type,address
fn gen_utxo_data_from_csv_line(line: &str) -> Result<UTXOData> {
    let str_list: Vec<&str> = line.trim().split(',').collect();
    if str_list.len() != 9 {
        return Err(Error::from(RoochError::from(Error::msg(format!(
            "Invalid csv line: {}",
            line
        )))));
    }
    let txid = str_list[1].to_string();
    let vout = str_list[2]
        .parse::<u32>()
        .map_err(|e| RoochError::from(Error::msg(format!("Invalid vout format: {}", e))))?;
    let amount = str_list[5]
        .parse::<u64>()
        .map_err(|e| RoochError::from(Error::msg(format!("Invalid amount format: {}", e))))?;
    let script = str_list[6].to_string();
    let script_type = str_list[7].to_string();
    let address = str_list[8].to_string();
    let utxo_data = UTXOData::new(txid, vout, amount, script, script_type, address.clone());
    if address.is_empty() && !utxo_data.is_valid_empty_address() {
        return Err(Error::from(RoochError::from(Error::msg(format!(
            "Invalid utxo data: {:?}",
            utxo_data
        )))));
    }
    Ok(utxo_data)
}

pub fn apply_utxo_updates_to_state(
    rx: Receiver<BatchUpdates>,
    moveos_store: Arc<MoveOSStore>,

    root_size: u64,
    root_state_root: H256,

    startup_update_set: Option<UpdateSet<FieldKey, ObjectState>>,

    task_start_time: SystemTime,
) {
    let moveos_store = &moveos_store.clone();
    let mut utxo_count = 0;
    let mut address_mapping_count = 0;

    let mut utxo_store_state_root = *GENESIS_STATE_ROOT;
    let mut rooch_to_bitcoin_address_mapping_state_root = *GENESIS_STATE_ROOT;

    let mut last_utxo_store_state_root = utxo_store_state_root;
    let mut last_rooch_to_bitcoin_address_mapping_state_root =
        rooch_to_bitcoin_address_mapping_state_root;

    while let Ok(batch) = rx.recv() {
        let loop_start_time = SystemTime::now();

        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();

        let cnt = batch.utxo_updates.len();
        let mut utxo_tree_change_set =
            apply_fields(moveos_store, utxo_store_state_root, batch.utxo_updates).unwrap();
        nodes.append(&mut utxo_tree_change_set.nodes);
        utxo_store_state_root = utxo_tree_change_set.state_root;
        utxo_count += cnt as u64;

        if !batch.rooch_to_bitcoin_mapping_updates.is_empty() {
            let cnt = batch.rooch_to_bitcoin_mapping_updates.len();
            let mut rooch_to_bitcoin_address_mapping_tree_change_set = apply_fields(
                moveos_store,
                rooch_to_bitcoin_address_mapping_state_root,
                batch.rooch_to_bitcoin_mapping_updates,
            )
            .unwrap();
            nodes.append(&mut rooch_to_bitcoin_address_mapping_tree_change_set.nodes);
            rooch_to_bitcoin_address_mapping_state_root =
                rooch_to_bitcoin_address_mapping_tree_change_set.state_root;
            address_mapping_count += cnt as u64;
        }

        apply_nodes(moveos_store, nodes).expect("failed to apply nodes");

        println!(
            "{} utxo, {} addr_mapping applied. This bacth cost: {:?}",
            // because we skip the first line, count result keep missing one.
            // e.g. batch_size = 8192:
            // 8191 utxo applied in: 1.000000000s
            // 16383 utxo applied in: 1.000000000s
            utxo_count,
            address_mapping_count,
            loop_start_time.elapsed().unwrap()
        );

        log::debug!(
            "last_utxo_store_state_root: {:?}, new utxo_store_state_root: {:?}; \
            last_rooch_to_bitcoin_address_mapping_state_root: {:?}, new rooch_to_bitcoin_address_mapping_state_root: {:?}",
            last_utxo_store_state_root,utxo_store_state_root,
            last_rooch_to_bitcoin_address_mapping_state_root,rooch_to_bitcoin_address_mapping_state_root
        );

        last_utxo_store_state_root = utxo_store_state_root;
        last_rooch_to_bitcoin_address_mapping_state_root =
            rooch_to_bitcoin_address_mapping_state_root;
    }

    finish_task(
        utxo_count,
        address_mapping_count,
        moveos_store,
        root_size,
        root_state_root,
        utxo_store_state_root,
        rooch_to_bitcoin_address_mapping_state_root,
        task_start_time,
        startup_update_set,
    );
}

fn finish_task(
    utxo_count: u64,
    address_mapping_count: u64,

    moveos_store: &MoveOSStore,

    root_size: u64,
    mut root_state_root: H256,
    utxo_store_state_root: H256,
    rooch_to_bitcoin_address_mapping_state_root: H256,

    task_start_time: SystemTime,
    startup_update_set: Option<UpdateSet<FieldKey, ObjectState>>,
) {
    // Update UTXOStore Object
    let mut genesis_utxostore_object = create_genesis_utxostore_object().unwrap();
    genesis_utxostore_object.size += utxo_count;
    genesis_utxostore_object.state_root = Some(utxo_store_state_root);
    let mut update_set = startup_update_set.unwrap_or_default();
    let parent_id = BitcoinUTXOStore::object_id();
    update_set.put(parent_id.field_key(), genesis_utxostore_object.into_state());

    // Update Address Mapping Object

    let mut genesis_rooch_to_bitcoin_address_mapping_object =
        create_genesis_rooch_to_bitcoin_address_mapping_object().unwrap();

    genesis_rooch_to_bitcoin_address_mapping_object.size += address_mapping_count;
    genesis_rooch_to_bitcoin_address_mapping_object.state_root =
        Some(rooch_to_bitcoin_address_mapping_state_root);

    update_set.put(
        genesis_rooch_to_bitcoin_address_mapping_object
            .id
            .field_key(),
        genesis_rooch_to_bitcoin_address_mapping_object.into_state(),
    );
    let tree_change_set = apply_fields(moveos_store, root_state_root, update_set).unwrap();
    apply_nodes(moveos_store, tree_change_set.nodes).unwrap();
    root_state_root = tree_change_set.state_root;

    // Update Startup Info
    let new_startup_info = StartupInfo::new(root_state_root, root_size);
    moveos_store
        .get_config_store()
        .save_startup_info(new_startup_info)
        .unwrap();

    let startup_info = moveos_store.get_config_store().get_startup_info().unwrap();
    println!(
        "Done in {:?}. New startup_info: {:?}",
        task_start_time.elapsed().unwrap(),
        startup_info
    );
}

fn create_genesis_utxostore_object() -> Result<ObjectEntity<BitcoinUTXOStore>> {
    let utxostore_object = BitcoinUTXOStore { next_tx_index: 0 };
    let utxostore_id = BitcoinUTXOStore::object_id();
    let utxostore_object = ObjectEntity::new(
        utxostore_id,
        SYSTEM_OWNER_ADDRESS,
        SHARED_OBJECT_FLAG_MASK,
        None,
        0,
        0,
        0,
        utxostore_object,
    );
    Ok(utxostore_object)
}

fn create_genesis_rooch_to_bitcoin_address_mapping_object(
) -> Result<ObjectEntity<RoochToBitcoinAddressMapping>> {
    let object_id = RoochToBitcoinAddressMapping::object_id();
    let reverse_address_mapping_object = ObjectEntity::new(
        object_id,
        SYSTEM_OWNER_ADDRESS,
        0u8,
        None,
        0,
        0,
        0,
        RoochToBitcoinAddressMapping::default(),
    );
    Ok(reverse_address_mapping_object)
}

struct AddressMappingUpdate {
    key: FieldKey,
    state: ObjectState,
}

pub struct BatchUpdates {
    utxo_updates: UpdateSet<FieldKey, ObjectState>,
    rooch_to_bitcoin_mapping_updates: UpdateSet<FieldKey, ObjectState>,
}

pub fn produce_utxo_updates(
    tx: SyncSender<BatchUpdates>,
    input: PathBuf,
    batch_size: usize,
    utxo_ord_map_db: Option<Arc<Database>>,
) {
    let file_cache_mgr = FileCacheManager::new(input.clone()).unwrap();
    let mut cache_drop_offset: u64 = 0;

    let mut csv_reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(input).unwrap());
    let mut is_title_line = true;
    let mut address_mapping_checker = HashMap::new();
    let utxo_ord_map = match utxo_ord_map_db {
        None => None,
        Some(utxo_ord_map_db) => {
            let read_txn = utxo_ord_map_db.begin_read().unwrap();
            Some(Arc::new(read_txn.open_table(UTXO_ORD_MAP_TABLE).unwrap()))
        }
    };
    loop {
        let mut bytes_read = 0;

        let mut updates = BatchUpdates {
            utxo_updates: UpdateSet::new(),
            rooch_to_bitcoin_mapping_updates: UpdateSet::new(),
        };
        for line in csv_reader.by_ref().lines().take(batch_size) {
            let line = line.unwrap();
            bytes_read += line.len() as u64 + 1; // Add line.len() + 1, assuming that the line terminator is '\n'

            if is_title_line {
                is_title_line = false;
                if line.starts_with("count") {
                    continue;
                }
            }

            let utxo_data = gen_utxo_data_from_csv_line(&line).unwrap();
            let (key, state, address_mapping_data) =
                match gen_utxo_update(utxo_data.clone(), utxo_ord_map.clone()) {
                    Ok((key, state, address_mapping_data)) => (key, state, address_mapping_data),
                    Err(e) => {
                        panic!(
                            "failed to gen_utxo_update: {:?} for {}[{:?}]",
                            e, line, utxo_data
                        );
                    }
                };
            updates.utxo_updates.put(key, state);

            if let Some(address_mapping_data) = address_mapping_data {
                let address_mapping_update =
                    gen_address_mapping_update(address_mapping_data, &mut address_mapping_checker);
                if let Some(address_mapping_update) = address_mapping_update {
                    updates
                        .rooch_to_bitcoin_mapping_updates
                        .put(address_mapping_update.key, address_mapping_update.state);
                }
            }
        }
        let _ = file_cache_mgr.drop_cache_range(cache_drop_offset, bytes_read);
        cache_drop_offset += bytes_read;
        if updates.utxo_updates.is_empty() {
            break;
        }
        tx.send(updates).expect("failed to send updates");
    }

    drop(tx);
}

fn gen_utxo_update(
    mut utxo_data: UTXOData,
    utxo_ord_map: Option<Arc<ReadOnlyTable<&[u8], &[u8]>>>,
) -> Result<(FieldKey, ObjectState, Option<AddressMappingData>)> {
    let raw_txid = Txid::from_str(utxo_data.txid.as_str())?;
    let txid = raw_txid.into_address();

    let mut address = BITCOIN_MOVE_ADDRESS;
    let mut address_mapping_data = None;

    // reserve utxo by default bitcoin and rooch address
    let (address, address_mapping_data) = if SCRIPT_TYPE_P2MS.eq(utxo_data.script_type.as_str())
        || SCRIPT_TYPE_NON_STANDARD.eq(utxo_data.script_type.as_str())
    {
        (address, address_mapping_data)
    } else {
        if SCRIPT_TYPE_P2PK.eq(utxo_data.script_type.as_str()) {
            if let Ok(pubkey) = PublicKey::from_str(utxo_data.script.as_str()) {
                let pubkey_hash = pubkey.pubkey_hash();
                let bitcoin_address = BitcoinAddress::new_p2pkh(&pubkey_hash);
                utxo_data.address = bitcoin_address.to_string();
            }
        }

        if let Ok(bitcoin_address) = BitcoinAddress::from_str(utxo_data.address.as_str()) {
            address = AccountAddress::from(bitcoin_address.to_rooch_address());
            address_mapping_data = Some(AddressMappingData::new(
                utxo_data.address.clone(),
                bitcoin_address,
                address,
            ));
        }

        (address, address_mapping_data)
    };

    let ids_in_seal = get_ord_by_outpoint(utxo_ord_map, OutPoint::new(raw_txid, utxo_data.vout));
    let seals = inscription_object_ids_to_utxo_seal(ids_in_seal);
    let utxo = UTXO::new(txid, utxo_data.vout, utxo_data.value, seals);
    let out_point = types::OutPoint::new(txid, utxo_data.vout);
    let utxo_id = utxo::derive_utxo_id(&out_point);
    let utxo_object = ObjectEntity::new(utxo_id, address, 0u8, None, 0, 0, 0, utxo);
    Ok((
        utxo_object.id.field_key(),
        utxo_object.into_state(),
        address_mapping_data,
    ))
}

fn inscription_object_ids_to_utxo_seal(
    obj_ids: Option<Vec<ObjectID>>,
) -> SimpleMultiMap<MoveString, ObjectID> {
    if let Some(obj_ids) = obj_ids {
        SimpleMultiMap {
            data: vec![Element {
                key: MoveString::from_str(UTXO_SEAL_INSCRIPTION_PROTOCOL).unwrap(),
                value: obj_ids,
            }],
        }
    } else {
        SimpleMultiMap::create()
    }
}

fn gen_address_mapping_update(
    address_mapping_data: AddressMappingData,
    address_mapping_checker: &mut HashMap<String, bool>,
) -> Option<AddressMappingUpdate> {
    if let Entry::Vacant(e) =
        address_mapping_checker.entry(address_mapping_data.origin_address.clone())
    {
        let state = address_mapping_data.into_state();
        let key = state.id().field_key();
        e.insert(true);

        return Some(AddressMappingUpdate { key, state });
    }
    None
}
