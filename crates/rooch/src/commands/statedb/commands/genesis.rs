// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::SystemTime;

use clap::Parser;

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::GENESIS_STATE_ROOT;
use moveos_types::state::{FieldKey, ObjectState};
use rooch_common::fs::file_cache::FileCacheManager;
use rooch_common::utils::humanize;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::bitcoin::ord::InscriptionStore;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use smt::UpdateSet;

use crate::cli_types::WalletContextOptions;
use crate::commands::statedb::commands::genesis_utxo::{
    apply_address_updates, apply_utxo_updates, produce_utxo_updates,
};
use crate::commands::statedb::commands::import::{apply_fields, apply_nodes, finish_import_job};
use crate::commands::statedb::commands::inscription::{
    create_genesis_inscription_store_object, gen_inscription_ids_update, InscriptionSource,
};
use crate::commands::statedb::commands::{init_job, OutpointInscriptionsMap};

/// Import BTC ordinals & UTXO for genesis
#[derive(Debug, Parser)]
pub struct GenesisCommand {
    #[clap(long, short = 'i')]
    /// utxo source data file. like ~/.rooch/local/utxo.csv or utxo.csv
    /// The file format is csv, and the first line is the header, the header is as follows:
    /// count,txid,vout,height,coinbase,amount,script,type,address
    pub utxo_source: PathBuf,
    #[clap(long)]
    /// ord source data file. like ~/.rooch/local/ord or ord, ord_input must be sorted by sequence_number
    /// The file format is json, and the first line is block height info: # export at block height <N>, ord range: [0, N).
    /// ord_input & utxo_input must be in the same height
    pub ord_source: PathBuf,
    #[clap(
        long,
        default_value = "1048576",
        help = "batch size submited to state db. Set it smaller if memory is limited."
    )]
    pub utxo_batch_size: Option<usize>,
    #[clap(
        long,
        default_value = "524288",
        help = "batch size submited to state db. Set it smaller if memory is limited."
    )] // ord may have large body, so set a smaller batch
    pub ord_batch_size: Option<usize>,
    #[clap(
        long,
        help = "outpoint(original):inscriptions(object_id) map dump path, for debug"
    )]
    pub outpoint_inscriptions_map_dump_path: Option<PathBuf>,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl GenesisCommand {
    // 1. init import job
    // 2. import ord (record utxo_seal)
    // 3. import utxo with utxo_seal
    // 4. update genesis
    // 5. print job stats, clean env
    pub async fn execute(self) -> RoochResult<()> {
        // 1. init import job
        let (root, moveos_store, start_time) =
            init_job(self.base_data_dir.clone(), self.chain_id.clone());
        let pre_root_state_root = root.state_root();

        log::info!("indexing and dumping outpoint_inscriptions_map...");
        let (outpoint_inscriptions_map, mapped_outpoint, mapped_inscription) =
            OutpointInscriptionsMap::index_and_dump(
                self.ord_source.clone(),
                self.outpoint_inscriptions_map_dump_path.clone(),
            );
        println!(
            "{} outpoints : {} inscriptions mapped in: {:?}",
            mapped_outpoint,
            mapped_inscription,
            start_time.elapsed(),
        );

        // import inscriptions and utxo parallel
        let outpoint_inscriptions_map = Arc::new(outpoint_inscriptions_map);
        let moveos_store = Arc::new(moveos_store);
        let startup_update_set = Arc::new(RwLock::new(UpdateSet::new()));
        // import inscriptions
        let input_path = self.ord_source.clone();
        let batch_size = self.ord_batch_size.unwrap();
        let (ord_tx, ord_rx) = mpsc::sync_channel(3);
        let produce_ord_updates_thread =
            thread::spawn(move || produce_inscription_updates(ord_tx, input_path, batch_size));
        let moveos_store_clone = Arc::clone(&moveos_store);
        let startup_update_set_clone = Arc::clone(&startup_update_set);
        let apply_ord_updates_thread = thread::spawn(move || {
            apply_inscription_updates(ord_rx, moveos_store_clone, startup_update_set_clone);
        });

        // import utxo
        let utxo_input_path = self.utxo_source.clone();
        let utxo_batch_size = self.utxo_batch_size.unwrap();
        let (utxo_tx, utxo_rx) = mpsc::sync_channel(4);
        let (addr_tx, addr_rx) = mpsc::sync_channel(2);
        let produce_utxo_updates_thread = thread::spawn(move || {
            produce_utxo_updates(
                utxo_tx,
                addr_tx,
                utxo_input_path,
                utxo_batch_size,
                Some(outpoint_inscriptions_map),
            )
        });
        let moveos_store_clone = Arc::clone(&moveos_store);
        let startup_update_set_clone = Arc::clone(&startup_update_set);
        let apply_addr_updates_thread = thread::spawn(move || {
            apply_address_updates(addr_rx, moveos_store_clone, startup_update_set_clone);
        });
        let moveos_store_clone = Arc::clone(&moveos_store);
        let startup_update_set_clone = Arc::clone(&startup_update_set);
        let apply_utxo_updates_thread = thread::spawn(move || {
            apply_utxo_updates(utxo_rx, moveos_store_clone, startup_update_set_clone);
        });

        produce_ord_updates_thread.join().unwrap();
        produce_utxo_updates_thread.join().unwrap();
        apply_ord_updates_thread.join().unwrap();
        apply_addr_updates_thread.join().unwrap();
        apply_utxo_updates_thread.join().unwrap();

        finish_import_job(
            Arc::clone(&moveos_store),
            root.size(),
            pre_root_state_root,
            start_time,
            Some(Arc::clone(&startup_update_set)),
        );

        Ok(())
    }
}

struct InscriptionUpdates {
    update_set: UpdateSet<FieldKey, ObjectState>,
    cursed_inscription_count: u32,
    blessed_inscription_count: u32,

    updates_value_bytes: u64, // stat for optimization
}

fn produce_inscription_updates(
    tx: SyncSender<InscriptionUpdates>,
    input: PathBuf,
    batch_size: usize,
) {
    let file_cache_mgr = FileCacheManager::new(input.clone()).unwrap();
    let mut src_reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(input).unwrap());
    let mut is_title_line = true;
    let mut sequence_number: u32 = 0;

    let mut cache_drop_offset: u64 = 0;
    loop {
        let mut bytes_read = 0;
        let mut updates = InscriptionUpdates {
            update_set: UpdateSet::new(),
            cursed_inscription_count: 0,
            blessed_inscription_count: 0,
            updates_value_bytes: 0,
        };
        let loop_start_time = SystemTime::now();

        for line in src_reader.by_ref().lines().take(batch_size) {
            let line = line.unwrap();
            bytes_read += line.len() as u64 + 1; // Add line.len() + 1, assuming that the line terminator is '\n'

            if is_title_line {
                is_title_line = false;
                if line.starts_with("# export at") {
                    // skip block height info
                    continue;
                }
            }

            let source: InscriptionSource = InscriptionSource::from_str(&line);
            if source.inscription_number < 0 {
                updates.cursed_inscription_count += 1;
            } else {
                updates.blessed_inscription_count += 1;
            }
            let (key, state, inscription_id) = source.gen_update();
            updates.updates_value_bytes += state.value.len() as u64;
            updates.update_set.put(key, state);
            let (key2, state2) = gen_inscription_ids_update(sequence_number, inscription_id);
            updates.update_set.put(key2, state2);
            sequence_number += 1;
        }
        println!(
            "{} inscription updates produced, cost: {:?}",
            updates.blessed_inscription_count + updates.cursed_inscription_count,
            loop_start_time.elapsed().unwrap()
        );
        let _ = file_cache_mgr.drop_cache_range(cache_drop_offset, bytes_read);
        cache_drop_offset += bytes_read;

        if updates.update_set.is_empty() {
            break;
        }

        tx.send(updates).expect("failed to send updates");
    }

    drop(tx);
}

fn apply_inscription_updates(
    rx: Receiver<InscriptionUpdates>,
    moveos_store_arc: Arc<MoveOSStore>,
    startup_update_set: Arc<RwLock<UpdateSet<FieldKey, ObjectState>>>,
) {
    let mut inscription_store_state_root = *GENESIS_STATE_ROOT;
    let mut last_inscription_store_state_root = inscription_store_state_root;
    let mut inscritpion_store_filed_count = 0u32;
    let mut cursed_inscription_count = 0u32;
    let mut blessed_inscription_count = 0u32;
    let moveos_store = moveos_store_arc.as_ref();
    while let Ok(batch) = rx.recv() {
        let loop_start_time = SystemTime::now();

        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();

        let cnt = batch.update_set.len();
        let mut ord_tree_change_set =
            apply_fields(moveos_store, inscription_store_state_root, batch.update_set).unwrap();
        nodes.append(&mut ord_tree_change_set.nodes);

        inscription_store_state_root = ord_tree_change_set.state_root;
        cursed_inscription_count += batch.cursed_inscription_count;
        blessed_inscription_count += batch.blessed_inscription_count;

        apply_nodes(moveos_store, nodes).expect("failed to apply ord nodes");

        inscritpion_store_filed_count += cnt as u32;

        println!(
            "{} inscription applied ({} cursed, {} blessed). this batch: value size: {}, cost: {:?}",
            inscritpion_store_filed_count / 2, // both ord and ord_id as field
            cursed_inscription_count,
            blessed_inscription_count,
            humanize::human_readable_bytes(batch.updates_value_bytes),
            loop_start_time.elapsed().unwrap()
        );

        log::debug!(
            "last inscription_store_state_root: {:?}, new inscription_store_state_root: {:?}",
            last_inscription_store_state_root,
            inscription_store_state_root,
        );

        last_inscription_store_state_root = inscription_store_state_root;
    }

    drop(rx);

    let mut startup_update_set = startup_update_set.write().unwrap();

    let mut genesis_inscription_store_object = create_genesis_inscription_store_object(
        cursed_inscription_count,
        blessed_inscription_count,
        inscritpion_store_filed_count / 2,
    );
    genesis_inscription_store_object.size += inscritpion_store_filed_count as u64;
    genesis_inscription_store_object.state_root = Some(inscription_store_state_root);
    let parent_id = InscriptionStore::object_id();
    startup_update_set.put(
        parent_id.field_key(),
        genesis_inscription_store_object.into_state(),
    );
    println!(
        "genesis InscriptionStore object updated, state_root: {:?}, cursed: {}, blessed: {}, total: {}",
        inscription_store_state_root, cursed_inscription_count, blessed_inscription_count, inscritpion_store_filed_count / 2
    );
}
