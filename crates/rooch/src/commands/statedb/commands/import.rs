// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::SystemTime;

use anyhow::{Error, Result};
use chrono::{DateTime, Local};
use clap::Parser;
use serde::{Deserialize, Serialize};

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{ObjectID, RootObjectEntity, GENESIS_STATE_ROOT};
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{KeyState, State};
use moveos_types::state_resolver::StatelessResolver;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::address_mapping::AddressMappingWrapper;
use rooch_types::rooch_network::RoochChainID;
use smt::{TreeChangeSet, UpdateSet};

use crate::commands::statedb::commands::export::ExportID;
use crate::commands::statedb::commands::{
    init_statedb, GLOBAL_STATE_TYPE_PREFIX, GLOBAL_STATE_TYPE_ROOT,
};

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
        let root_state_root = H256::from(root.state_root.into_bytes());
        let (tx, rx) = mpsc::sync_channel(2);

        let moveos_store_arc = Arc::new(moveos_store.clone());
        let produce_updates_thread = thread::spawn(move || {
            produce_updates(tx, &moveos_store, input_path, root_state_root, batch_size)
        });
        let apply_updates_thread = thread::spawn(move || {
            apply_updates_to_state(rx, moveos_store_arc, root_state_root, root.size, start_time)
        });
        let _ = produce_updates_thread
            .join()
            .map_err(|_e| RoochError::from(Error::msg("Produce updates error".to_string())))?;
        let _ = apply_updates_thread
            .join()
            .map_err(|_e| RoochError::from(Error::msg("Apply updates error ".to_string())))?;

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

struct BatchUpdates {
    states: BTreeMap<StateID, UpdateSet<KeyState, State>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct StateID {
    pub export_id: ExportID,
    // start state root
    pub pre_state_root: H256,
}

impl StateID {
    pub fn new(export_id: ExportID, pre_state_root: H256) -> Self {
        Self {
            export_id,
            pre_state_root,
        }
    }
}

fn produce_updates(
    tx: SyncSender<BatchUpdates>,
    moveos_store: &MoveOSStore,
    input: PathBuf,
    root_state_root: H256,
    batch_size: usize,
) -> Result<()> {
    let mut csv_reader = BufReader::new(File::open(input).unwrap());
    let mut last_state_id = None;
    loop {
        let mut updates = BatchUpdates {
            states: BTreeMap::new(),
        };
        for line in csv_reader.by_ref().lines().take(batch_size) {
            let line = line?;

            if line.starts_with(GLOBAL_STATE_TYPE_PREFIX) {
                // TODO check current statedb root state root
                if line.starts_with(GLOBAL_STATE_TYPE_ROOT) {
                    break;
                }
                let (_c1, c2) = parse_state_data_from_csv_line(&line)?;
                let export_id = ExportID::from_str(&c2)?;
                // let eventual_state_root = export_id.parent_state_root;
                // // TODO add cache to avoid duplicate read smt
                let pre_state_root =
                    get_pre_state_root(moveos_store, root_state_root, export_id.object_id.clone())?;

                let state_id = StateID::new(export_id, pre_state_root);
                updates.states.insert(state_id.clone(), UpdateSet::new());
                last_state_id = Some(state_id);
                continue;
            }

            let (c1, c2) = parse_state_data_from_csv_line(&line)?;
            let key_state = KeyState::from_str(&c1)?;
            let state = State::from_str(&c2)?;
            let state_id = last_state_id.clone().expect("State ID should have value");
            let update_set = updates.states.entry(state_id).or_default();
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

// csv format: c1,c2
pub fn parse_state_data_from_csv_line(line: &str) -> Result<(String, String)> {
    let str_list: Vec<&str> = line.trim().split(',').collect();
    if str_list.len() != 2 {
        return Err(Error::from(RoochError::from(Error::msg(format!(
            "Invalid csv line: {}",
            line
        )))));
    }
    let c1 = str_list[0].to_string();
    let c2 = str_list[1].to_string();
    Ok((c1, c2))
}

fn apply_updates_to_state(
    rx: Receiver<BatchUpdates>,
    moveos_store: Arc<MoveOSStore>,
    root_state_root: H256,
    root_size: u64,
    task_start_time: SystemTime,
) -> Result<()> {
    // let mut _count = 0;
    let mut last_state_root = root_state_root;
    while let Ok(batch) = rx.recv() {
        let loop_start_time = SystemTime::now();

        for (state_id, update_set) in batch.states.into_iter() {
            let mut tree_change_set =
                apply_fields(&moveos_store, state_id.pre_state_root, update_set)?;
            let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();
            nodes.append(&mut tree_change_set.nodes);
            last_state_root = tree_change_set.state_root;

            apply_nodes(&moveos_store, nodes).expect("failed to apply nodes");

            log::debug!(
                "state_root: {:?}, new state_root: {:?} execpt state_root: {:?}",
                state_id.pre_state_root,
                last_state_root,
                state_id.export_id.state_root
            );
        }

        println!("This bacth cost: {:?}", loop_start_time.elapsed().unwrap());
    }

    finish_task(&moveos_store, last_state_root, root_size, task_start_time);
    Ok(())
}

fn finish_task(
    moveos_store: &MoveOSStore,
    root_state_root: H256,
    root_size: u64,
    task_start_time: SystemTime,
) {
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

/// Get current pre state root of object id for import node
pub fn get_pre_state_root(
    moveos_store: &MoveOSStore,
    root_state_root: H256,
    // state_root: H256,
    // parent_state_root: H256,
    object_id: ObjectID,
) -> Result<H256> {
    let parent_state_root_opt = match object_id.parent() {
        Some(parent_id) => {
            let state_opt = moveos_store.get_field_at(root_state_root, &parent_id.to_key())?;
            match state_opt {
                Some(state) => Some(H256::from(
                    state.clone().as_raw_object()?.state_root.into_bytes(),
                )),
                None => None,
            }
        }
        None => Some(root_state_root),
    };
    let state_root = match parent_state_root_opt {
        Some(parent_state_root) => {
            let state_opt = moveos_store.get_field_at(parent_state_root, &object_id.to_key())?;
            match state_opt {
                Some(state) => H256::from(state.as_raw_object()?.state_root.into_bytes()),
                None => *GENESIS_STATE_ROOT,
            }
        }
        None => *GENESIS_STATE_ROOT,
    };

    Ok(state_root)
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

pub fn init_import_job(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> (RootObjectEntity, MoveOSStore, SystemTime) {
    let start_time = SystemTime::now();
    let datetime: DateTime<Local> = start_time.into();

    let (root, moveos_store) = init_statedb(base_data_dir.clone(), chain_id.clone()).unwrap();
    let utxo_store_id = BitcoinUTXOStore::object_id();
    let address_mapping_id = AddressMappingWrapper::mapping_object_id();
    let reverse_mapping_object_id = AddressMappingWrapper::reverse_mapping_object_id();
    let inscription_store_id = InscriptionStore::object_id();

    println!("task progress started at {}", datetime,);
    println!("root object: {:?}", root);
    println!("utxo_store_id: {:?}", utxo_store_id);
    println!(
        "address_mapping_id: {:?}, reverse_mapping_object_id {:?}",
        address_mapping_id, reverse_mapping_object_id
    );
    println!("inscription_store_id: {:?}", inscription_store_id);
    (root, moveos_store, start_time)
}
