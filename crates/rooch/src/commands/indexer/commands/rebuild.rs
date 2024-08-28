// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Instant;

use anyhow::{Error, Result};
use clap::Parser;

use moveos_types::state::ObjectState;
use rooch_config::R_OPT_NET_HELP;
use rooch_indexer::indexer_reader::IndexerReader;
use rooch_indexer::store::traits::IndexerStoreTrait;
use rooch_indexer::IndexerStore;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::indexer::state::{
    IndexerObjectState, IndexerObjectStateChangeSet, IndexerObjectStatesIndexGenerator,
    ObjectStateType,
};
use rooch_types::rooch_network::RoochChainID;

use crate::commands::indexer::commands::init_indexer;
use crate::commands::statedb::commands::import::parse_csv_fields;

/// Rebuild indexer
#[derive(Debug, Parser)]
pub struct RebuildCommand {
    #[clap(long, short = 'i')]
    /// import an input file. like ~/.rooch/local/indexer.csv or indexer.csv
    pub input: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data would be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long, short = 'b', default_value = "20971")]
    pub batch_size: Option<usize>,
}

impl RebuildCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let input_path = self.input.clone();
        let batch_size = self.batch_size.unwrap();
        let (indexer_store, indexer_reader, start_time) = self.init();
        let (tx, rx) = mpsc::sync_channel(2);

        let produce_updates_thread = tokio::task::spawn_blocking(move || {
            produce_updates(tx, indexer_reader, input_path, batch_size)
        });
        let apply_updates_thread =
            tokio::task::spawn_blocking(move || apply_updates(rx, indexer_store, start_time));
        let _ = produce_updates_thread
            .await
            .map_err(|_e| RoochError::from(Error::msg("Produce updates error".to_string())))?;
        let _ = apply_updates_thread
            .await
            .map_err(|_e| RoochError::from(Error::msg("Apply updates error ".to_string())))?;

        Ok(())
    }

    fn init(self) -> (IndexerStore, IndexerReader, Instant) {
        let start_time = Instant::now();
        let (indexer_store, indexer_reader) =
            init_indexer(self.base_data_dir.clone(), self.chain_id.clone()).unwrap();
        log::info!("indexer rebuild started");
        (indexer_store, indexer_reader, start_time)
    }
}

struct BatchUpdates {
    object_state_change_set: IndexerObjectStateChangeSet,
}

fn produce_updates(
    tx: SyncSender<BatchUpdates>,
    indexer_reader: IndexerReader,
    input: PathBuf,
    batch_size: usize,
) -> Result<()> {
    let mut csv_reader = BufReader::new(File::open(input).unwrap());

    // set genesis tx_order and state_index_generator for indexer rebuild
    let tx_order: u64 = 0;
    let state_index_start = indexer_reader
        .query_last_state_index_by_tx_order(tx_order, ObjectStateType::ObjectState)?
        .map_or(0, |x| x + 1);
    let utxo_state_index_start = indexer_reader
        .query_last_state_index_by_tx_order(tx_order, ObjectStateType::UTXO)?
        .map_or(0, |x| x + 1);
    let inscription_state_index_start = indexer_reader
        .query_last_state_index_by_tx_order(tx_order, ObjectStateType::Inscription)?
        .map_or(0, |x| x + 1);
    let mut state_index_generator = IndexerObjectStatesIndexGenerator {
        object_states_index_generator: state_index_start,
        object_state_utxos_index_generator: utxo_state_index_start,
        object_state_inscriptions_generator: inscription_state_index_start,
    };

    println!(
        "Indexer rebuild, state_index_generator: {:?} ",
        state_index_generator
    );

    loop {
        let mut updates = BatchUpdates {
            object_state_change_set: IndexerObjectStateChangeSet::default(),
        };

        for line in csv_reader.by_ref().lines().take(batch_size) {
            let line = line?;

            let (_c1, state_str) = parse_csv_fields(&line)?;
            let state_result = ObjectState::from_str(&state_str);
            match state_result {
                Ok(state) => {
                    let object_type = state.metadata.object_type.clone();
                    let state_index = state_index_generator.get(&object_type);
                    let indexer_state =
                        IndexerObjectState::new(state.metadata, tx_order, state_index);
                    updates
                        .object_state_change_set
                        .new_object_states(indexer_state);
                    state_index_generator.incr(&object_type);
                }
                Err(e) => {
                    println!(
                        "Parse object state error, state maybe not an object, line: {} , err: {:?}",
                        line,
                        e.to_string()
                    );
                    continue;
                }
            }
        }
        if updates
            .object_state_change_set
            .object_states
            .new_object_states
            .is_empty()
            && updates
                .object_state_change_set
                .object_state_utxos
                .new_object_states
                .is_empty()
            && updates
                .object_state_change_set
                .object_state_inscriptions
                .new_object_states
                .is_empty()
        {
            break;
        }
        tx.send(updates).expect("failed to send updates");
    }

    drop(tx);
    Ok(())
}

fn apply_updates(
    rx: Receiver<BatchUpdates>,
    indexer_store: IndexerStore,
    task_start_time: Instant,
) -> Result<()> {
    let mut ok_count: usize = 0;
    while let Ok(batch) = rx.recv() {
        let loop_start_time = Instant::now();
        let object_states_len = batch
            .object_state_change_set
            .object_states
            .new_object_states
            .len();
        let utxos_len = batch
            .object_state_change_set
            .object_state_utxos
            .new_object_states
            .len();
        let inscriptions_len = batch
            .object_state_change_set
            .object_state_inscriptions
            .new_object_states
            .len();
        let count = object_states_len + utxos_len + inscriptions_len;
        indexer_store.persist_or_update_object_states(
            batch
                .object_state_change_set
                .object_states
                .new_object_states,
        )?;
        indexer_store.persist_or_update_object_state_utxos(
            batch
                .object_state_change_set
                .object_state_utxos
                .new_object_states,
        )?;
        indexer_store.persist_or_update_object_state_inscriptions(
            batch
                .object_state_change_set
                .object_state_inscriptions
                .new_object_states,
        )?;
        ok_count += count;
        println!(
            "Total {} updates applied. this batch process object states count {}, utxo count {}, inscription count {}. this batch cost: {:?}",
            ok_count,
            object_states_len,
            utxos_len,
            inscriptions_len,
            loop_start_time.elapsed()
        );
    }

    println!(
        "Indexer rebuild task finished({} updates applied) in: {:?}",
        ok_count,
        task_start_time.elapsed()
    );

    Ok(())
}
