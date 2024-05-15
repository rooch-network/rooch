// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
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

use anyhow::{Error, Result};
use bitcoin::{PublicKey, Txid};
use chrono::{DateTime, Local};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use serde::{Deserialize, Serialize};

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{
    ObjectEntity, ObjectID, RootObjectEntity, GENESIS_STATE_ROOT, SHARED_OBJECT_FLAG_MASK,
    SYSTEM_OWNER_ADDRESS,
};
use moveos_types::moveos_std::simple_multimap::SimpleMultiMap;
use moveos_types::moveos_std::table::TablePlaceholder;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{KeyState, MoveState, MoveType, State};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::address::{BitcoinAddress, MultiChainAddress, RoochAddress};
use rooch_types::addresses::BITCOIN_MOVE_ADDRESS;
use rooch_types::bitcoin::utxo::{BitcoinUTXOStore, UTXO};
use rooch_types::bitcoin::{types, utxo};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::address_mapping::AddressMappingWrapper;
use rooch_types::into_address::IntoAddress;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::rooch_network::RoochChainID;
use smt::{TreeChangeSet, UpdateSet};

use crate::commands::statedb::commands::export::ExportID;
use crate::commands::statedb::commands::{init_statedb, STATE_HEADER_PREFIX};

/// Import statedb
#[derive(Debug, Parser)]
pub struct ImportCommand {
    #[clap(long, short = 'i')]
    /// import input file. like ~/.rooch/local/utxo.csv or utxo.csv
    pub input: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long, short = 'b', default_value = "20971")]
    pub batch_size: Option<usize>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl ImportCommand {
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
                start_time,
            );
        });
        produce_updates_thread.join()?;
        apply_updates_thread.join()?;

        Ok(())
    }

    fn init(self) -> (RootObjectEntity, MoveOSStore, SystemTime) {
        let start_time = SystemTime::now();
        let datetime: DateTime<Local> = start_time.into();

        let (root, moveos_store) =
            init_statedb(self.base_data_dir.clone(), self.chain_id.clone()).unwrap();

        println!(
            "task progress started at {}, batch_size: {}",
            datetime,
            self.batch_size.unwrap()
        );
        println!("root object: {:?}", root);
        (root, moveos_store, start_time)
    }
}

// csv format: c1,c2
fn parse_state_data_from_csv_line(line: &str) -> Result<(String, String)> {
    let str_list: Vec<&str> = line.trim().split(',').collect();
    if str_list.len() != 2 {
        return Err(Error::from(RoochError::from(Error::msg(format!(
            "Invalid csv line: {}",
            line
        )))));
    }
    let c1 = str_list[1].to_string();
    let c2 = str_list[1].to_string();
    Ok((c1, c2))
}

fn apply_updates_to_state(
    rx: Receiver<BatchUpdates>,
    moveos_store: &MoveOSStore,

    root_size: u64,
    root_state_root: H256,

    mut state_root: H256,

    task_start_time: SystemTime,
) {
    let mut state_count = 0;
    // let mut address_mapping_count = 0;

    let mut last_state_root = state_root;

    while let Ok(batch) = rx.recv() {
        let loop_start_time = SystemTime::now();

        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();

        let cnt = batch.state_updates.len();
        let mut utxo_tree_change_set =
            apply_fields(moveos_store, state_root, batch.state_updates).unwrap();
        nodes.append(&mut utxo_tree_change_set.nodes);
        state_root = utxo_tree_change_set.state_root;
        state_count += cnt as u64;

        apply_nodes(moveos_store, nodes).expect("failed to apply nodes");

        println!(
            "{} utxo, {} addr_mapping applied. This bacth cost: {:?}",
            // because we skip the first line, count result keep missing one.
            // e.g. batch_size = 8192:
            // 8191 utxo applied in: 1.000000000s
            // 16383 utxo applied in: 1.000000000s
            state_count,
            address_mapping_count,
            loop_start_time.elapsed().unwrap()
        );

        log::debug!(
            "last_state_root: {:?}, new state_root: {:?}; "
            last_state_root, state_root,
        );

        last_state_root = state_root;
    }

    finish_task(
        state_count,
        address_mapping_count,
        moveos_store,
        root_size,
        root_state_root,
        state_root,
        address_mapping_state_root,
        reverse_address_mapping_state_root,
        task_start_time,
    );
}

fn finish_task(
    state_count: u64,
    address_mapping_count: u64,

    moveos_store: &MoveOSStore,

    root_size: u64,
    mut root_state_root: H256,
    state_root: H256,
    address_mapping_state_root: H256,
    reverse_address_mapping_state_root: H256,

    task_start_time: SystemTime,
) {
    // Update UTXOStore Object
    let mut genesis_utxostore_object = create_genesis_utxostore_object().unwrap();
    genesis_utxostore_object.size += state_count;
    genesis_utxostore_object.state_root = state_root.into_address();
    let mut update_set = UpdateSet::new();
    let parent_id = BitcoinUTXOStore::object_id();
    update_set.put(parent_id.to_key(), genesis_utxostore_object.into_state());

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

struct BatchUpdates {
    states: BTreeMap<StateRootKey, UpdateSet<KeyState, State>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct StateRootKey {
    pub object_id: ObjectID,
    // start state root
    pub state_root: H256,
    // eventual expect state root
    pub eventual_state_root: H256,
}

impl StateRootKey {
    pub fn new(object_id: ObjectID, state_root: H256, eventual_state_root: H256) -> Self {
        Self {
            object_id,
            state_root,
            eventual_state_root,
        }
    }
}

fn produce_updates(
    tx: SyncSender<BatchUpdates>,
    input: PathBuf,
    // last_state_root: Option<H256>,
    batch_size: usize,
) -> Result() {
    let mut csv_reader = BufReader::new(File::open(input).unwrap());
    // let mut state_root = last_state_root.unwrap_or(H256::zero());
    loop {
        let mut updates = BatchUpdates {
            states: BTreeMap::new(),
        };
        for line in csv_reader.by_ref().lines().take(batch_size) {
            let line = line?;

            if line.starts_with(STATE_HEADER_PREFIX) {
                let (c1, c2) = parse_state_data_from_csv_line(&line)?;
                let export_id = ExportID::from_str(&c1)?;
                let eventual_state_root = H256::from_str(&c2)?;
                let state_root_key = StateRootKey::new(export_id.object_id, eventual_state_root);
                updates.states.insert(state_root_key, UpdateSet::new());
                continue;
            }

            let (c1, c2) = parse_state_data_from_csv_line(&line)?;
            let key_state = KeyState::from_str(&c1)?;
            let state = State::from_str(&c2)?;
            // let (key, state, address_mapping_data) = gen_state_update(utxo_data).unwrap();
            let update_set = updates.states.entry(state_root_key).or_insert(UpdateSet::new());
            update_set.put(key_state, state);
        }
        if updates.states.is_empty() {
            break;
        }
        tx.send(updates).expect("failed to send updates");
    }

    drop(tx);
    Ok(())
}

pub fn apply_fields<I>(
    moveos_store: &MoveOSStore,
    pre_state_root: H256,
    update_set: I,
) -> Result<TreeChangeSet>
where
    I: Into<UpdateSet<KeyState, State>>,
{
    let tree_change_set = moveos_store
        .statedb
        .update_fields(pre_state_root, update_set)?;
    Ok(tree_change_set)
}

pub fn apply_nodes(moveos_store: &MoveOSStore, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
    moveos_store.statedb.node_store.write_nodes(nodes)?;
    Ok(())
}
