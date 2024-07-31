// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;
use std::time::SystemTime;

use clap::Parser;
use rustc_hash::FxHashSet;
use tokio::time::Instant;

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::GENESIS_STATE_ROOT;
use moveos_types::state::{FieldKey, ObjectState};
use rooch_common::fs::file_cache::FileCacheManager;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::utxo::BitcoinUTXOStore;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use smt::UpdateSet;

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::{init_job, OutpointInscriptionsMap};
use crate::commands::statedb::commands::import::{apply_fields, apply_nodes, finish_import_job};
use crate::commands::statedb::commands::utxo::{
    create_genesis_rooch_to_bitcoin_address_mapping_object, create_genesis_utxo_store_object,
    UTXORawData,
};

/// Import UTXO for development and testing.
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

    #[clap(long, short = 'b', default_value = "524288")]
    pub batch_size: Option<usize>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl GenesisUTXOCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (root, moveos_store, start_time) =
            init_job(self.base_data_dir.clone(), self.chain_id.clone());
        let root_size = root.size;
        let pre_root_state_root = root.state_root();
        let startup_update_set = Arc::new(RwLock::new(UpdateSet::new()));
        let moveos_store_arc = Arc::new(moveos_store);

        let (utxo_tx, utxo_rx) = mpsc::sync_channel(4);
        let (addr_tx, addr_rx) = mpsc::sync_channel(4);
        let produce_updates_thread = thread::spawn(move || {
            produce_utxo_updates(
                utxo_tx,
                addr_tx,
                self.input.clone(),
                self.batch_size.unwrap(),
                None,
            )
        });
        let moveos_store_clone = Arc::clone(&moveos_store_arc);
        let startup_update_set_clone = Arc::clone(&startup_update_set);
        let apply_addr_updates_thread = thread::spawn(move || {
            apply_address_updates(addr_rx, moveos_store_clone, startup_update_set_clone);
        });
        let moveos_store_clone = Arc::clone(&moveos_store_arc);
        let startup_update_set_clone = Arc::clone(&startup_update_set);
        let apply_utxo_updates_thread = thread::spawn(move || {
            apply_utxo_updates(utxo_rx, moveos_store_clone, startup_update_set_clone);
        });
        produce_updates_thread.join().unwrap();
        apply_addr_updates_thread.join().unwrap();
        apply_utxo_updates_thread.join().unwrap();

        finish_import_job(
            Arc::clone(&moveos_store_arc),
            root_size,
            pre_root_state_root,
            start_time,
            Some(Arc::clone(&startup_update_set)),
        );
        Ok(())
    }
}

pub(crate) fn produce_utxo_updates(
    utxo_tx: SyncSender<UpdateSet<FieldKey, ObjectState>>,
    addr_tx: SyncSender<UpdateSet<FieldKey, ObjectState>>,
    input: PathBuf,
    batch_size: usize,
    outpoint_inscriptions_map: Option<Arc<OutpointInscriptionsMap>>,
) {
    let file_cache_mgr = FileCacheManager::new(input.clone()).unwrap();
    let mut cache_drop_offset: u64 = 0;
    let mut reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(input).unwrap());
    let mut is_title_line = true;
    let mut added_address_set: FxHashSet<String> =
        FxHashSet::with_capacity_and_hasher(60_000_000, Default::default());
    let mut max_height = 0;

    loop {
        let loop_start_time = Instant::now();
        let mut bytes_read = 0;
        let mut utxo_updates = UpdateSet::new();
        let mut rooch_to_bitcoin_mapping_updates = UpdateSet::new();

        for line in reader.by_ref().lines().take(batch_size) {
            let line = line.unwrap();
            bytes_read += line.len() as u64 + 1; // Add line.len() + 1, assuming that the line terminator is '\n'

            if is_title_line {
                is_title_line = false;
                if line.starts_with("count") {
                    continue;
                }
            }

            let mut utxo_raw = UTXORawData::from_str(&line);
            let (key, state, address_mapping_data) =
                utxo_raw.gen_update(outpoint_inscriptions_map.clone());
            utxo_updates.put(key, state);
            if utxo_raw.height > max_height {
                max_height = utxo_raw.height;
            }

            if let Some(address_mapping_data) = address_mapping_data {
                if let Some((field_key, object_state)) =
                    address_mapping_data.gen_update(&mut added_address_set)
                {
                    rooch_to_bitcoin_mapping_updates.put(field_key, object_state);
                }
            }
        }
        println!(
            "{} utxo + {} addr_mapping updates produced, cost: {:?}",
            utxo_updates.len(),
            rooch_to_bitcoin_mapping_updates.len(),
            loop_start_time.elapsed(),
        );
        let _ = file_cache_mgr.drop_cache_range(cache_drop_offset, bytes_read);
        cache_drop_offset += bytes_read;

        if !rooch_to_bitcoin_mapping_updates.is_empty() {
            addr_tx
                .send(rooch_to_bitcoin_mapping_updates)
                .expect("failed to send updates");
        }
        if utxo_updates.is_empty() {
            break;
        }
        utxo_tx.send(utxo_updates).expect("failed to send updates");
    }

    drop(utxo_tx);
    drop(addr_tx);
    println!("utxo max_height: {}", max_height);
}

pub(crate) fn apply_address_updates(
    rx: Receiver<UpdateSet<FieldKey, ObjectState>>,
    moveos_store: Arc<MoveOSStore>,
    startup_update_set: Arc<RwLock<UpdateSet<FieldKey, ObjectState>>>,
) {
    let mut address_mapping_count = 0;
    let mut rooch_to_bitcoin_address_mapping_state_root = *GENESIS_STATE_ROOT;
    let mut last_rooch_to_bitcoin_address_mapping_state_root =
        rooch_to_bitcoin_address_mapping_state_root;

    while let Ok(update_set) = rx.recv() {
        let loop_start_time = SystemTime::now();

        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();
        let cnt = update_set.len();
        let mut rooch_to_bitcoin_address_mapping_tree_change_set = apply_fields(
            &moveos_store,
            rooch_to_bitcoin_address_mapping_state_root,
            update_set,
        )
        .unwrap();
        nodes.append(&mut rooch_to_bitcoin_address_mapping_tree_change_set.nodes);
        rooch_to_bitcoin_address_mapping_state_root =
            rooch_to_bitcoin_address_mapping_tree_change_set.state_root;
        address_mapping_count += cnt as u64;

        apply_nodes(&moveos_store, nodes).expect("failed to apply nodes");

        println!(
            "{} addr_mapping applied. this batch: {}, cost: {:?}",
            address_mapping_count,
            cnt,
            loop_start_time.elapsed().unwrap()
        );

        log::debug!(
            "last_rooch_to_bitcoin_address_mapping_state_root: {:?}, new rooch_to_bitcoin_address_mapping_state_root: {:?}",
            last_rooch_to_bitcoin_address_mapping_state_root,rooch_to_bitcoin_address_mapping_state_root
        );

        last_rooch_to_bitcoin_address_mapping_state_root =
            rooch_to_bitcoin_address_mapping_state_root;
    }

    let mut genesis_rooch_to_bitcoin_address_mapping_object =
        create_genesis_rooch_to_bitcoin_address_mapping_object();
    genesis_rooch_to_bitcoin_address_mapping_object.size += address_mapping_count;
    genesis_rooch_to_bitcoin_address_mapping_object.state_root =
        Some(rooch_to_bitcoin_address_mapping_state_root);

    let mut startup_update_set = startup_update_set.write().unwrap();
    startup_update_set.put(
        genesis_rooch_to_bitcoin_address_mapping_object
            .id
            .field_key(),
        genesis_rooch_to_bitcoin_address_mapping_object.into_state(),
    );
    println!(
        "genesis RoochToBitcoinAddressMapping object updated, state_root: {:?}, count: {}",
        rooch_to_bitcoin_address_mapping_state_root, address_mapping_count
    );
}

pub(crate) fn apply_utxo_updates(
    rx: Receiver<UpdateSet<FieldKey, ObjectState>>,
    moveos_store: Arc<MoveOSStore>,
    startup_update_set: Arc<RwLock<UpdateSet<FieldKey, ObjectState>>>,
) {
    let moveos_store = &moveos_store.clone();
    let mut utxo_count = 0;

    let mut utxo_store_state_root = *GENESIS_STATE_ROOT;

    let mut last_utxo_store_state_root = utxo_store_state_root;

    while let Ok(update_set) = rx.recv() {
        let loop_start_time = SystemTime::now();

        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();

        let cnt = update_set.len();
        let mut utxo_tree_change_set =
            apply_fields(moveos_store, utxo_store_state_root, update_set).unwrap();
        nodes.append(&mut utxo_tree_change_set.nodes);
        utxo_store_state_root = utxo_tree_change_set.state_root;
        utxo_count += cnt as u64;

        apply_nodes(moveos_store, nodes).expect("failed to apply nodes");

        println!(
            "{} utxo applied. this bacth: {}, cost: {:?}",
            // because we may skip the first line of data source, count result keep missing one.
            // e.g. batch_size = 8192:
            // 8191 utxo applied ...
            utxo_count,
            cnt,
            loop_start_time.elapsed().unwrap()
        );

        log::debug!(
            "last_utxo_store_state_root: {:?}, new utxo_store_state_root: {:?}",
            last_utxo_store_state_root,
            utxo_store_state_root,
        );

        last_utxo_store_state_root = utxo_store_state_root;
    }

    let mut startup_update_set = startup_update_set.write().unwrap();

    let mut genesis_utxostore_object = create_genesis_utxo_store_object();
    genesis_utxostore_object.size += utxo_count;
    genesis_utxostore_object.state_root = Some(utxo_store_state_root);
    startup_update_set.put(
        BitcoinUTXOStore::object_id().field_key(),
        genesis_utxostore_object.into_state(),
    );
    println!(
        "genesis BitcoinUTXOStore object updated, state_root: {:?}, count: {}",
        utxo_store_state_root, utxo_count
    );
}
