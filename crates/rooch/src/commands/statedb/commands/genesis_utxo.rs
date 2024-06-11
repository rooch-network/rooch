// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::import::{apply_fields, apply_nodes};
use anyhow::{Error, Result};
use bitcoin::{PublicKey, Txid};
use chrono::{DateTime, Local};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{
    ObjectEntity, RootObjectEntity, GENESIS_STATE_ROOT, SHARED_OBJECT_FLAG_MASK,
    SYSTEM_OWNER_ADDRESS,
};
use moveos_types::moveos_std::simple_multimap::SimpleMultiMap;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{KeyState, MoveState, State};
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
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
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
        let pre_root_state_root = H256::from(root.state_root.into_bytes());
        let (tx, rx) = mpsc::sync_channel(2);

        let produce_updates_thread =
            thread::spawn(move || produce_updates(tx, input_path, batch_size));
        let apply_updates_thread = thread::spawn(move || {
            apply_updates_to_state(
                rx,
                &moveos_store,
                root.size,
                pre_root_state_root,
                *GENESIS_STATE_ROOT,
                *GENESIS_STATE_ROOT,
                start_time,
            );
        });
        produce_updates_thread.join().unwrap();
        apply_updates_thread.join().unwrap();

        Ok(())
    }

    fn init(self) -> (RootObjectEntity, MoveOSStore, SystemTime) {
        let start_time = SystemTime::now();
        let datetime: DateTime<Local> = start_time.into();

        let opt = RoochOpt::new_with_default(self.base_data_dir, self.chain_id, None).unwrap();
        let rooch_db = RoochDB::init(opt.store_config()).unwrap();
        let (root, moveos_store) = (rooch_db.root, rooch_db.moveos_store);

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
        (root, moveos_store, start_time)
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

fn apply_updates_to_state(
    rx: Receiver<BatchUpdates>,
    moveos_store: &MoveOSStore,

    root_size: u64,
    root_state_root: H256,

    mut utxo_store_state_root: H256,
    mut rooch_to_bitcoin_address_mapping_state_root: H256,

    task_start_time: SystemTime,
) {
    let mut utxo_count = 0;
    let mut address_mapping_count = 0;

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
) {
    // Update UTXOStore Object
    let mut genesis_utxostore_object = create_genesis_utxostore_object().unwrap();
    genesis_utxostore_object.size += utxo_count;
    genesis_utxostore_object.state_root = utxo_store_state_root.into_address();
    let mut update_set = UpdateSet::new();
    let parent_id = BitcoinUTXOStore::object_id();
    update_set.put(parent_id.to_key(), genesis_utxostore_object.into_state());

    // Update Address Mapping Object

    let mut genesis_rooch_to_bitcoin_address_mapping_object =
        create_genesis_rooch_to_bitcoin_address_mapping_object().unwrap();

    genesis_rooch_to_bitcoin_address_mapping_object.size += address_mapping_count;
    genesis_rooch_to_bitcoin_address_mapping_object.state_root =
        rooch_to_bitcoin_address_mapping_state_root.into_address();

    update_set.put(
        genesis_rooch_to_bitcoin_address_mapping_object.id.to_key(),
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
        *GENESIS_STATE_ROOT,
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
        *GENESIS_STATE_ROOT,
        0,
        0,
        0,
        RoochToBitcoinAddressMapping::default(),
    );
    Ok(reverse_address_mapping_object)
}

struct AddressMappingUpdate {
    key: KeyState,
    state: State,
}

struct BatchUpdates {
    utxo_updates: UpdateSet<KeyState, State>,
    rooch_to_bitcoin_mapping_updates: UpdateSet<KeyState, State>,
}

fn produce_updates(tx: SyncSender<BatchUpdates>, input: PathBuf, batch_size: usize) {
    let mut csv_reader = BufReader::new(File::open(input).unwrap());
    let mut is_title_line = true;
    let mut address_mapping_checker = HashMap::new();
    loop {
        let mut updates = BatchUpdates {
            utxo_updates: UpdateSet::new(),
            rooch_to_bitcoin_mapping_updates: UpdateSet::new(),
        };
        for line in csv_reader.by_ref().lines().take(batch_size) {
            let line = line.unwrap();

            if is_title_line {
                is_title_line = false;
                if line.starts_with("count") {
                    continue;
                }
            }

            let utxo_data = gen_utxo_data_from_csv_line(&line).unwrap();
            let (key, state, address_mapping_data) = gen_utxo_update(utxo_data).unwrap();
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
        if updates.utxo_updates.is_empty() {
            break;
        }
        tx.send(updates).expect("failed to send updates");
    }

    drop(tx);
}

fn gen_utxo_update(
    mut utxo_data: UTXOData,
) -> Result<(KeyState, State, Option<AddressMappingData>)> {
    let txid = Txid::from_str(utxo_data.txid.as_str())?.into_address();

    // reserve utxo by default bitcoin and rooch address
    let (address, address_mapping_data) = if SCRIPT_TYPE_P2MS.eq(utxo_data.script_type.as_str())
        || SCRIPT_TYPE_NON_STANDARD.eq(utxo_data.script_type.as_str())
    {
        let _bitcoin_address = BitcoinAddress::default();
        let address = BITCOIN_MOVE_ADDRESS;
        (address, None)
    } else {
        if SCRIPT_TYPE_P2PK.eq(utxo_data.script_type.as_str()) {
            let pubkey = PublicKey::from_str(utxo_data.script.as_str())?;
            let pubkey_hash = pubkey.pubkey_hash();
            let bitcoin_address = BitcoinAddress::new_p2pkh(&pubkey_hash);
            utxo_data.address = bitcoin_address.to_string();
        }

        let bitcoin_address = BitcoinAddress::from_str(utxo_data.address.as_str())?;
        let address = AccountAddress::from(bitcoin_address.to_rooch_address());
        let address_mapping_data =
            AddressMappingData::new(utxo_data.address, bitcoin_address, address);
        (address, Some(address_mapping_data))
    };
    let utxo = UTXO::new(
        txid,
        utxo_data.vout,
        utxo_data.value,
        SimpleMultiMap::create(),
    );
    let out_point = types::OutPoint::new(txid, utxo_data.vout);
    let utxo_id = utxo::derive_utxo_id(&out_point);
    let utxo_object = ObjectEntity::new(utxo_id, address, 0u8, *GENESIS_STATE_ROOT, 0, 0, 0, utxo);
    Ok((
        utxo_object.id.to_key(),
        utxo_object.into_state(),
        address_mapping_data,
    ))
}

fn gen_address_mapping_update(
    address_mapping_data: AddressMappingData,
    address_mapping_checker: &mut HashMap<String, bool>,
) -> Option<AddressMappingUpdate> {
    if let Entry::Vacant(e) = address_mapping_checker.entry(address_mapping_data.origin_address) {
        let key = KeyState::from_address(address_mapping_data.address);
        let state = address_mapping_data.baddress.into_state();

        e.insert(true);

        return Some(AddressMappingUpdate { key, state });
    }
    None
}
