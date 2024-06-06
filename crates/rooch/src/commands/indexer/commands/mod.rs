// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use rooch_config::RoochOpt;
use rooch_indexer::indexer_reader::IndexerReader;
use rooch_indexer::IndexerStore;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

pub mod rebuild;

pub const BATCH_SIZE: usize = 5000;
fn init_indexer(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> Result<(IndexerStore, IndexerReader)> {
    // Reconstruct RoochOpt
    let opt = RoochOpt::new_with_default(base_data_dir, chain_id, None)?;

    let store_config = opt.store_config();

    let indexer_db_path = store_config.get_indexer_store_dir();
    let indexer_store = IndexerStore::new(indexer_db_path.clone())?;
    let indexer_reader = IndexerReader::new(indexer_db_path)?;

    Ok((indexer_store, indexer_reader))
}
