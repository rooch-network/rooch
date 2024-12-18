// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use metrics::RegistryService;
use moveos_types::moveos_std::object::ObjectMeta;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_types::da::chunk::chunk_from_segments;
use rooch_types::da::segment::{segment_from_bytes, SegmentID};
use rooch_types::rooch_network::RoochChainID;
use rooch_types::transaction::LedgerTransaction;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub mod exec;
pub mod get_tx_order_hash;
pub mod namespace;
pub mod unpack;

// collect all the chunks from segment_dir.
// each segment is stored in a file named by the segment_id.
// each chunk may contain multiple segments.
// we collect all the chunks and their segment numbers to unpack them later.
pub(crate) fn collect_chunks(
    segment_dir: PathBuf,
) -> anyhow::Result<(HashMap<u128, Vec<u64>>, u128, u128)> {
    let mut chunks = HashMap::new();
    let mut max_chunk_id = 0;
    let mut min_chunk_id = u128::MAX;
    for entry in fs::read_dir(segment_dir.clone())?.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(segment_id) = path
                .file_name()
                .and_then(|s| s.to_str()?.parse::<SegmentID>().ok())
            {
                let chunk_id = segment_id.chunk_id;
                let segment_number = segment_id.segment_number;
                let segments: &mut Vec<u64> = chunks.entry(chunk_id).or_default();
                segments.push(segment_number);
                if chunk_id > max_chunk_id {
                    max_chunk_id = chunk_id;
                }
                if chunk_id < min_chunk_id {
                    min_chunk_id = chunk_id;
                }
            }
        }
    }
    if chunks.is_empty() {
        return Err(anyhow::anyhow!("No segment found in {:?}", segment_dir));
    }
    Ok((chunks, min_chunk_id, max_chunk_id))
}

pub(crate) fn get_tx_list_from_chunk(
    segment_dir: PathBuf,
    chunk_id: u128,
    segment_numbers: Vec<u64>,
) -> anyhow::Result<Vec<LedgerTransaction>> {
    let mut segments = Vec::new();
    for segment_number in segment_numbers {
        let segment_id = SegmentID {
            chunk_id,
            segment_number,
        };
        let segment_path = segment_dir.join(segment_id.to_string());
        let segment_bytes = fs::read(segment_path)?;
        let segment = segment_from_bytes(&segment_bytes)?;
        segments.push(segment);
    }
    let chunk = chunk_from_segments(segments)?;
    let batch = chunk.get_batches().into_iter().next().unwrap();
    batch.verify(true)?;
    Ok(batch.get_tx_list())
}

pub(crate) fn build_rooch_db(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
    enable_rocks_stats: bool,
) -> (ObjectMeta, RoochDB) {
    let mut opt = RoochOpt::new_with_default(base_data_dir, chain_id, None).unwrap();
    opt.store.enable_statistics = enable_rocks_stats;
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();
    let root = rooch_db.latest_root().unwrap().unwrap();
    (root, rooch_db)
}

pub struct LedgerTxLoader {
    segment_dir: PathBuf,
    chunks: HashMap<u128, Vec<u64>>,
    min_chunk_id: u128,
    max_chunk_id: u128,
}

impl LedgerTxLoader {
    pub fn new(segment_dir: PathBuf) -> anyhow::Result<Self> {
        let (chunks, min_chunk_id, max_chunk_id) = collect_chunks(segment_dir.clone())?;

        Ok(LedgerTxLoader {
            segment_dir,
            chunks,
            min_chunk_id,
            max_chunk_id,
        })
    }

    pub fn load_ledger_tx_list(
        &self,
        block_number: u128,
    ) -> anyhow::Result<Option<Vec<LedgerTransaction>>> {
        let segments = self.chunks.get(&block_number);
        if segments.is_none() {
            return Ok(None);
        }
        let tx_list = get_tx_list_from_chunk(
            self.segment_dir.clone(),
            block_number,
            segments.unwrap().clone(),
        )?;
        Ok(Some(tx_list))
    }

    pub fn get_max_chunk_id(&self) -> u128 {
        self.max_chunk_id
    }

    pub fn get_min_chunk_id(&self) -> u128 {
        self.min_chunk_id
    }
}
