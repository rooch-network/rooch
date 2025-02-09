// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::statedb::commands::inscription::{
    gen_inscription_id_update, InscriptionSource, InscriptionStats,
};
use crate::commands::statedb::commands::{apply_fields, apply_nodes, init_rooch_db};
use clap::Parser;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::GENESIS_STATE_ROOT;
use moveos_types::state::{FieldKey, ObjectState};
use rooch_common::fs::FileCacheManager;
use rooch_common::humanize;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use smt::UpdateSet;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Instant, SystemTime};

/// Import BTC Inscription only for genesis in development and testing env
#[derive(Debug, Parser)]
pub struct GenesisOrdCommand {
    #[clap(long)]
    /// ord source data file. like ~/.rooch/local/ord or ord, ord_input must be sorted by sequence_number
    pub ord_source: PathBuf,
    #[clap(long)]
    /// ord stats file, like ~/.rooch/local/ord_stats or ord_stats
    pub ord_stats: PathBuf,
    #[clap(
        long,
        default_value = "1048576",
        help = "batch size submitted to state db. Set it smaller if memory is limited."
    )] // ord may have a large body, so set a smaller batch
    pub ord_batch_size: Option<usize>,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GenesisOrdCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let rooch_db = init_rooch_db(self.base_data_dir.clone(), self.chain_id.clone());
        let moveos_store = rooch_db.moveos_store;
        let moveos_store = Arc::new(moveos_store);

        let inscription_stats = InscriptionStats::load_from_file(self.ord_stats.clone());
        let input_path = self.ord_source.clone();
        let batch_size = self.ord_batch_size.unwrap();
        let (ord_tx, ord_rx) = mpsc::sync_channel(2);
        let produce_inscription_updates_thread =
            thread::spawn(move || produce_inscription_updates(ord_tx, input_path, batch_size));
        let moveos_store_clone = Arc::clone(&moveos_store);
        let apply_inscription_updates_thread = thread::spawn(move || {
            apply_inscription_updates(ord_rx, moveos_store_clone, inscription_stats);
        });

        produce_inscription_updates_thread.join().unwrap();
        apply_inscription_updates_thread.join().unwrap();

        Ok(())
    }
}

pub(crate) struct InscriptionUpdates {
    update_set: UpdateSet<FieldKey, ObjectState>,
    cursed_inscription_count: u32,
    blessed_inscription_count: u32,

    updates_value_bytes: u64, // stat for optimization
}

pub(crate) fn produce_inscription_updates(
    tx: SyncSender<InscriptionUpdates>,
    input: PathBuf,
    batch_size: usize,
) {
    let file_cache_mgr = FileCacheManager::new(input.clone()).unwrap();
    let mut src_reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(input).unwrap());
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
            let source: InscriptionSource = InscriptionSource::from_str(&line);
            if source.inscription_number < 0 {
                updates.cursed_inscription_count += 1;
            } else {
                updates.blessed_inscription_count += 1;
            }
            let (key, state, inscription_id) = source.gen_update();
            updates.updates_value_bytes += state.value.len() as u64;
            updates.update_set.put(key, state);
            let (key2, state2) = gen_inscription_id_update(sequence_number, inscription_id);
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

pub(crate) fn apply_inscription_updates(
    rx: Receiver<InscriptionUpdates>,
    moveos_store_arc: Arc<MoveOSStore>,
    exp_stats: InscriptionStats,
) {
    let mut inscription_store_state_root = *GENESIS_STATE_ROOT;
    let mut last_inscription_store_state_root = inscription_store_state_root;
    let mut inscritpion_store_field_count = 0u32;
    let mut cursed_inscription_count = 0u32;
    let mut blessed_inscription_count = 0u32;
    let moveos_store = moveos_store_arc.as_ref();
    while let Ok(batch) = rx.recv() {
        let loop_start_time = SystemTime::now();

        let gen_nodes_start = Instant::now();
        let mut nodes: BTreeMap<H256, Vec<u8>> = BTreeMap::new();

        let cnt = batch.update_set.len();

        let mut tree_change_set =
            apply_fields(moveos_store, inscription_store_state_root, batch.update_set).unwrap();
        nodes.append(&mut tree_change_set.nodes);
        let gen_nodes_cost = gen_nodes_start.elapsed();

        inscription_store_state_root = tree_change_set.state_root;
        cursed_inscription_count += batch.cursed_inscription_count;
        blessed_inscription_count += batch.blessed_inscription_count;

        let apply_nodes_start = Instant::now();
        apply_nodes(moveos_store, nodes).expect("failed to apply inscription nodes");
        let apply_nodes_cost = apply_nodes_start.elapsed();

        inscritpion_store_field_count += cnt as u32;

        println!(
            "{} inscription applied ({} cursed, {} blessed). this batch: value size: {}, cost: {:?}(gen_nodes: {:?}, apply_nodes: {:?})",
            inscritpion_store_field_count / 2, // both inscription and inscription_id as field
            cursed_inscription_count,
            blessed_inscription_count,
            humanize::human_readable_bytes(batch.updates_value_bytes),
            loop_start_time.elapsed().unwrap(),
            gen_nodes_cost,
            apply_nodes_cost,
        );

        tracing::debug!(
            "last inscription_store_state_root: {:?}, new inscription_store_state_root: {:?}",
            last_inscription_store_state_root,
            inscription_store_state_root,
        );

        last_inscription_store_state_root = inscription_store_state_root;
    }

    drop(rx);

    println!(
        "genesis InscriptionStore object updated, state_root: {:?}, cursed: {}, blessed: {}, total: {}",
        inscription_store_state_root, cursed_inscription_count, blessed_inscription_count, inscritpion_store_field_count / 2
    );

    let act_stats = InscriptionStats {
        block_height: exp_stats.block_height,
        cursed_inscription_count,
        blessed_inscription_count,
        unbound_inscription_count: exp_stats.unbound_inscription_count,
        lost_sats: exp_stats.lost_sats,
        next_sequence_number: inscritpion_store_field_count / 2,
    };
    assert_eq!(act_stats, exp_stats, "Inscription stats not match");
}
