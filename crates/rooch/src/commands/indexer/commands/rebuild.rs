// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;
use std::time::SystemTime;

use anyhow::{Error, Result};
use chrono::{DateTime, Local};
use clap::Parser;

use moveos_types::state::{KeyState, State};
use rooch_config::R_OPT_NET_HELP;
use rooch_indexer::indexer_reader::IndexerReader;
use rooch_indexer::store::traits::IndexerStoreTrait;
use rooch_indexer::IndexerStore;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::indexer::state::{IndexerFieldState, IndexerObjectState};
use rooch_types::rooch_network::RoochChainID;

use crate::commands::indexer::commands::init_indexer;
use crate::commands::statedb::commands::export::ExportID;
use crate::commands::statedb::commands::import::parse_state_data_from_csv_line;
use crate::commands::statedb::commands::{
    GLOBAL_STATE_TYPE_FIELD, GLOBAL_STATE_TYPE_OBJECT, GLOBAL_STATE_TYPE_PREFIX,
    GLOBAL_STATE_TYPE_ROOT,
};

/// Rebuild indexer
#[derive(Debug, Parser)]
pub struct RebuildCommand {
    #[clap(long, short = 'i')]
    /// import input file. like ~/.rooch/local/indexer.csv or indexer.csv
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

impl RebuildCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let input_path = self.input.clone();
        let batch_size = self.batch_size.unwrap();
        let (indexer_store, _indexer_reader, start_time) = self.init();
        let (tx, rx) = mpsc::sync_channel(2);

        let produce_updates_thread =
            thread::spawn(move || produce_updates(tx, input_path, batch_size));
        let apply_updates_thread =
            thread::spawn(move || apply_updates(rx, indexer_store, start_time));
        let _ = produce_updates_thread
            .join()
            .map_err(|_e| RoochError::from(Error::msg("Produce updates error".to_string())))?;
        let _ = apply_updates_thread
            .join()
            .map_err(|_e| RoochError::from(Error::msg("Apply updates error ".to_string())))?;

        Ok(())
    }

    fn init(self) -> (IndexerStore, IndexerReader, SystemTime) {
        let start_time = SystemTime::now();
        let datetime: DateTime<Local> = start_time.into();

        let (indexer_store, indexer_reader) =
            init_indexer(self.base_data_dir.clone(), self.chain_id.clone()).unwrap();

        println!(
            "indexer task progress started at {}, batch_size: {}",
            datetime,
            self.batch_size.unwrap()
        );
        (indexer_store, indexer_reader, start_time)
    }
}

struct BatchUpdates {
    object_states: Vec<IndexerObjectState>,
    field_states: Vec<IndexerFieldState>,
}

fn produce_updates(tx: SyncSender<BatchUpdates>, input: PathBuf, batch_size: usize) -> Result<()> {
    let mut csv_reader = BufReader::new(File::open(input).unwrap());
    let mut last_state_type = None;
    let mut last_export_id = None;

    // set genesis tx_order and state_index_generator
    let tx_order: u64 = 0;
    let mut state_index_generator: u64 = 0;

    loop {
        let mut updates = BatchUpdates {
            object_states: vec![],
            field_states: vec![],
        };

        for line in csv_reader.by_ref().lines().take(batch_size) {
            let line = line?;

            if line.starts_with(GLOBAL_STATE_TYPE_PREFIX) {
                let (c1, c2) = parse_state_data_from_csv_line(&line)?;
                let state_type = c1;
                let export_id = ExportID::from_str(&c2)?;
                last_state_type = Some(state_type);
                last_export_id = Some(export_id);
                continue;
            }

            let (c1, c2) = parse_state_data_from_csv_line(&line)?;
            let key_state = KeyState::from_str(&c1)?;
            let state = State::from_str(&c2)?;

            let state_type = last_state_type
                .clone()
                .expect("Last state type should have value");
            let export_id = last_export_id
                .clone()
                .expect("Last export id should have value");

            if state_type.eq(GLOBAL_STATE_TYPE_OBJECT) || state_type.eq(GLOBAL_STATE_TYPE_ROOT) {
                let raw_object = state.as_raw_object()?;
                let state = IndexerObjectState::try_new_from_state(
                    tx_order,
                    state_index_generator,
                    raw_object,
                )?;
                state_index_generator += 1;
                updates.object_states.push(state);
            } else if state_type.eq(GLOBAL_STATE_TYPE_FIELD) {
                let state = IndexerFieldState::new_field_state(
                    key_state,
                    state,
                    export_id.object_id.clone(),
                    tx_order,
                    state_index_generator,
                    export_id.timestamp,
                    true,
                );
                state_index_generator += 1;
                updates.field_states.push(state);
            };
        }
        if updates.object_states.is_empty() && updates.field_states.is_empty() {
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
    _task_start_time: SystemTime,
) -> Result<()> {
    while let Ok(batch) = rx.recv() {
        let loop_start_time = SystemTime::now();
        indexer_store.persist_or_update_object_states(batch.object_states)?;
        indexer_store.persist_or_update_field_states(batch.field_states)?;

        println!("This bacth cost: {:?}", loop_start_time.elapsed().unwrap());
    }

    Ok(())
}
