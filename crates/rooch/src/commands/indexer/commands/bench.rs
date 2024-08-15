// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;

use moveos_types::state::MoveStructType;
use rooch_config::R_OPT_NET_HELP;
use rooch_indexer::indexer_reader::IndexerReader;
use rooch_indexer::IndexerStore;
use rooch_types::bitcoin::ord::Inscription;
use rooch_types::bitcoin::utxo::UTXO;
use rooch_types::error::RoochResult;
use rooch_types::indexer::state::{IndexerStateID, ObjectStateFilter};
use rooch_types::rooch_network::RoochChainID;

use crate::commands::indexer::commands::init_indexer;

#[derive(Debug, Parser)]
pub struct BenchCommand {
    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    #[clap(long, help = "bench count", default_value = "1000000")]
    pub count: Option<u64>,

    #[clap(long, help = "query filter: utxo/ord", default_value = "utxo")]
    pub query_filter: Option<String>,

    /// If local chainid, start the service with a temporary data store.
    /// All data would be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl BenchCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (_, indexer_reader, _) = self.init();
        Self::count_cost_query(
            indexer_reader,
            self.query_filter.unwrap(),
            self.count.unwrap(),
        )
        .unwrap();
        Ok(())
    }

    fn init(&self) -> (IndexerStore, IndexerReader, Instant) {
        let start_time = Instant::now();
        let (indexer_store, indexer_reader) =
            init_indexer(self.base_data_dir.clone(), self.chain_id.clone()).unwrap();
        log::info!("indexer bench started");
        (indexer_store, indexer_reader, start_time)
    }

    fn count_cost_query(
        indexer_reader: IndexerReader,
        query_filter: String,
        count: u64,
    ) -> anyhow::Result<()> {
        const BATCH_SIZE: usize = 5000;

        let filter = match query_filter.as_str() {
            "utxo" => ObjectStateFilter::ObjectType(UTXO::struct_tag()),
            "ord" => ObjectStateFilter::ObjectType(Inscription::struct_tag()),
            _ => ObjectStateFilter::ObjectType(UTXO::struct_tag()),
        };

        let query_object_states =
            indexer_reader.query_object_states_with_filter(filter.clone(), None, 1, true)?;
        let tx_order = query_object_states[0].tx_order;
        let mut state_index = query_object_states[0].state_index;
        state_index += 1;
        let mut total_duration: u128 = 0;
        let mut total_query: u64 = 0;
        loop {
            if total_query >= count {
                break;
            }
            let start = Instant::now();
            for _ in 0..BATCH_SIZE {
                let query_object_states = indexer_reader.query_object_states_with_filter(
                    filter.clone(),
                    Some(IndexerStateID::new(tx_order, state_index)),
                    1,
                    true,
                )?;
                assert!(query_object_states.len() == 1);
                total_query += 1;
                state_index += 1;
            }
            let duration = start.elapsed().as_millis();
            total_duration += duration;
        }

        println!(
            "total query: {}, avg_duration(ms): {:.2},",
            total_query,
            total_duration as f64 / total_query as f64
        );

        Ok(())
    }
}
