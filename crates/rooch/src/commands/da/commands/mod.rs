// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use accumulator::{Accumulator, MerkleAccumulator};
use anyhow::anyhow;
use heed::byteorder::BigEndian;
use heed::types::{SerdeBincode, U64};
use heed::{Database, Env, EnvOpenOptions};
use metrics::RegistryService;
use moveos_store::transaction_store::{TransactionDBStore, TransactionStore};
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_common::vec::find_last_true;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_store::RoochStore;
use rooch_types::da::chunk::chunk_from_segments;
use rooch_types::da::segment::{segment_from_bytes, SegmentID};
use rooch_types::rooch_network::RoochChainID;
use rooch_types::sequencer::SequencerInfo;
use rooch_types::transaction::{LedgerTransaction, TransactionSequenceInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use tracing::info;

pub mod exec;
pub mod index;
pub mod namespace;
pub mod unpack;

pub(crate) struct SequencedTxStore {
    tx_accumulator: MerkleAccumulator,
    last_sequenced_tx_order: AtomicU64,
    rooch_store: RoochStore,
}

impl SequencedTxStore {
    pub(crate) fn new(rooch_store: RoochStore) -> anyhow::Result<Self> {
        // The sequencer info would be initialized when genesis, so the sequencer info should not be None
        let last_sequencer_info = rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("Load sequencer info failed"))?;
        let (last_order, last_accumulator_info) = (
            last_sequencer_info.last_order,
            last_sequencer_info.last_accumulator_info.clone(),
        );
        info!("Load latest sequencer order {:?}", last_order);
        info!(
            "Load latest sequencer accumulator info {:?}",
            last_accumulator_info
        );
        let tx_accumulator = MerkleAccumulator::new_with_info(
            last_accumulator_info,
            rooch_store.get_transaction_accumulator_store(),
        );

        Ok(SequencedTxStore {
            tx_accumulator,
            last_sequenced_tx_order: AtomicU64::new(last_order),
            rooch_store,
        })
    }

    pub(crate) fn get_last_tx_order(&self) -> u64 {
        self.last_sequenced_tx_order
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn store_tx(
        &self,
        mut tx: LedgerTransaction,
        exp_accumulator_root: Option<H256>,
    ) -> anyhow::Result<()> {
        let tx_order = tx.sequence_info.tx_order;
        match self.last_sequenced_tx_order.compare_exchange(
            tx_order - 1,
            tx_order,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        ) {
            Ok(_) => {
                // CAS succeeded, continue with function logic
            }
            Err(current) => {
                return Err(anyhow::anyhow!(
                    "CAS failed: Tx order is not strictly incremental. \
                Expected: {}, Actual: {}, Tx Order: {}",
                    tx_order - 1,
                    current,
                    tx_order
                ));
            }
        }

        let tx_hash = tx.tx_hash();
        let _tx_accumulator_root = self.tx_accumulator.append(vec![tx_hash].as_slice())?;
        let tx_accumulator_unsaved_nodes = self.tx_accumulator.pop_unsaved_nodes();
        let tx_accumulator_info = self.tx_accumulator.get_info();

        if let Some(exp_accumulator_root) = exp_accumulator_root {
            if tx_accumulator_info.accumulator_root != exp_accumulator_root {
                return Err(anyhow::anyhow!(
                    "Tx accumulator root mismatch, expect: {:?}, actual: {:?}",
                    exp_accumulator_root,
                    tx_accumulator_info.accumulator_root
                ));
            } else {
                info!(
                    "Accumulator root is equal to RoochNetwork: tx_order: {}",
                    tx_order
                );
            }
        }

        let sequencer_info = SequencerInfo::new(tx_order, tx_accumulator_info);
        self.rooch_store.save_sequenced_tx(
            tx_hash,
            tx.clone(),
            sequencer_info,
            tx_accumulator_unsaved_nodes,
        )
    }
}

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
    row_cache_size: Option<u64>,
    block_cache_size: Option<u64>,
) -> (ObjectMeta, RoochDB) {
    let mut opt = RoochOpt::new_with_default(base_data_dir, chain_id, None).unwrap();
    opt.store.enable_statistics = enable_rocks_stats;
    opt.store.row_cache_size = row_cache_size;
    opt.store.block_cache_size = block_cache_size;
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();
    let root = rooch_db.latest_root().unwrap().unwrap();
    (root, rooch_db)
}

pub struct LedgerTxGetter {
    segment_dir: PathBuf,
    chunks: HashMap<u128, Vec<u64>>,
    min_chunk_id: u128,
    max_chunk_id: u128,
}

impl LedgerTxGetter {
    pub fn new(segment_dir: PathBuf) -> anyhow::Result<Self> {
        let (chunks, min_chunk_id, max_chunk_id) = collect_chunks(segment_dir.clone())?;

        Ok(LedgerTxGetter {
            segment_dir,
            chunks,
            min_chunk_id,
            max_chunk_id,
        })
    }

    pub fn load_ledger_tx_list(
        &self,
        chunk_id: u128,
        must_has: bool,
    ) -> anyhow::Result<Option<Vec<LedgerTransaction>>> {
        let segments = self.chunks.get(&chunk_id);
        if segments.is_none() {
            if must_has {
                return Err(anyhow::anyhow!("No segment found in chunk {}", chunk_id));
            }
            return Ok(None);
        }
        let tx_list = get_tx_list_from_chunk(
            self.segment_dir.clone(),
            chunk_id,
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

#[derive(Debug, Clone)]
pub struct TxDAIndex {
    pub tx_order: u64,
    pub tx_hash: H256,
    pub block_number: u128,
}

impl TxDAIndex {
    pub fn new(tx_order: u64, tx_hash: H256, block_number: u128) -> Self {
        TxDAIndex {
            tx_order,
            tx_hash,
            block_number,
        }
    }
}

impl std::fmt::Display for TxDAIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{:?}:{}",
            self.tx_order, self.tx_hash, self.block_number
        )
    }
}

impl std::str::FromStr for TxDAIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid format"));
        }
        let tx_order = parts[0].parse::<u64>()?;
        let tx_hash = H256::from_str(parts[1])?;
        let block_number = parts[2].parse::<u128>()?;
        Ok(TxDAIndex {
            tx_order,
            tx_hash,
            block_number,
        })
    }
}

/// TxOrderHashBlockGetter is used to get TxOrderHashBlock from a file
/// all tx_order_hash_blocks(start from tx_order 1) are stored in a file,
/// each line is a TxOrderHashBlock
pub struct TxDAIndexer {
    tx_order_hash_blocks: Vec<TxDAIndex>,
    transaction_store: TransactionDBStore,
    rooch_store: RoochStore,
}

impl TxDAIndexer {
    pub fn load_from_file(
        file_path: PathBuf,
        transaction_store: TransactionDBStore,
        rooch_store: RoochStore,
    ) -> anyhow::Result<Self> {
        let mut tx_order_hashes = Vec::with_capacity(70000000);
        let mut reader = BufReader::new(File::open(file_path)?);
        for line in reader.by_ref().lines() {
            let line = line?;
            let item = line.parse::<TxDAIndex>()?;
            tx_order_hashes.push(item);
        }
        tx_order_hashes.sort_by(|a, b| a.tx_order.cmp(&b.tx_order)); // avoiding wrong order
        info!(
            "tx_order:tx_hash:block indexer loaded, tx cnt: {}",
            tx_order_hashes.len()
        );
        Ok(TxDAIndexer {
            tx_order_hash_blocks: tx_order_hashes,
            transaction_store,
            rooch_store,
        })
    }

    pub fn get_tx_hash(&self, tx_order: u64) -> Option<H256> {
        let r = self
            .tx_order_hash_blocks
            .binary_search_by(|x| x.tx_order.cmp(&tx_order));
        let idx = match r {
            Ok(i) => i,
            Err(_) => {
                return None;
            }
        };
        Some(self.tx_order_hash_blocks[idx].tx_hash)
    }

    pub fn slice(&self, start_tx_order: u64, end_tx_order: u64) -> anyhow::Result<Vec<TxDAIndex>> {
        let r = self
            .tx_order_hash_blocks
            .binary_search_by(|x| x.tx_order.cmp(&start_tx_order));
        let start_idx = match r {
            Ok(i) => i,
            Err(_) => {
                return Err(anyhow::anyhow!("start_tx_order not found"));
            }
        };
        let end_idx = start_idx + (end_tx_order - start_tx_order) as usize;
        Ok(self.tx_order_hash_blocks[start_idx..end_idx + 1].to_vec())
    }

    pub fn find_last_executed(&self) -> anyhow::Result<Option<TxDAIndex>> {
        let r = find_last_true(&self.tx_order_hash_blocks, |item| {
            self.has_executed(item.tx_hash)
        });
        Ok(r.cloned())
    }

    pub fn find_tx_block(&self, tx_order: u64) -> Option<u128> {
        let r = self
            .tx_order_hash_blocks
            .binary_search_by(|x| x.tx_order.cmp(&tx_order));
        let idx = match r {
            Ok(i) => i,
            Err(_) => {
                return None;
            }
        };
        Some(self.tx_order_hash_blocks[idx].block_number)
    }

    fn has_executed(&self, tx_hash: H256) -> bool {
        self.get_execution_info(tx_hash)
            .map_or(false, |info| info.is_some())
    }

    pub fn get_execution_info(
        &self,
        tx_hash: H256,
    ) -> anyhow::Result<Option<TransactionExecutionInfo>> {
        self.transaction_store.get_tx_execution_info(tx_hash)
    }

    pub fn get_sequencer_info(
        &self,
        tx_hash: H256,
    ) -> anyhow::Result<Option<TransactionSequenceInfo>> {
        Ok(self
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)?
            .map(|transaction| transaction.sequence_info))
    }
}

const MAP_SIZE: usize = 1 << 34; // 16G
const MAX_DBS: u32 = 1;
const ORDER_DATABASE_NAME: &str = "order_db";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TxPosition {
    pub tx_hash: H256,
    pub block_number: u128,
}

pub struct Indexer {
    db_env: Env,
    db: Database<U64<BigEndian>, SerdeBincode<TxPosition>>,
    last_tx_order: u64,
    last_block_number: u128,
}

#[derive(Debug, Serialize)]
pub struct IndexerStats {
    pub total_tx_count: u64,
    pub last_tx_order: u64,
    pub last_block_number: u128,
}

impl Indexer {
    pub fn load_or_dump(db_path: PathBuf, file_path: PathBuf, dump: bool) -> anyhow::Result<()> {
        if dump {
            let indexer = Indexer::new(db_path, None)?;
            indexer.dump_to_file(file_path)
        } else {
            Indexer::load_from_file(db_path, file_path)
        }
    }

    pub fn dump_to_file(&self, file_path: PathBuf) -> anyhow::Result<()> {
        let db = self.db;
        let file = std::fs::File::create(file_path)?;
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone().unwrap());
        let rtxn = self.db_env.read_txn()?;
        let mut iter = db.iter(&rtxn)?;
        while let Some((k, v)) = iter.next().transpose()? {
            writeln!(writer, "{}:{:?}:{}", k, v.tx_hash, v.block_number)?;
        }
        drop(iter);
        rtxn.commit()?;
        writer.flush().expect("Unable to flush writer");
        file.sync_data().expect("Unable to sync file");
        Ok(())
    }

    pub fn load_from_file(db_path: PathBuf, file_path: PathBuf) -> anyhow::Result<()> {
        let mut last_tx_order = 0;
        let mut last_tx_hash = H256::zero();
        let mut last_block_number = 0;

        let db_env = Self::create_env(db_path.clone())?;
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);

        let mut wtxn = db_env.write_txn()?; // Begin write_transaction early for create/put

        let mut is_verify = false;
        let db: Database<U64<BigEndian>, SerdeBincode<TxPosition>> =
            match db_env.open_database(&wtxn, Some(ORDER_DATABASE_NAME)) {
                Ok(Some(db)) => {
                    info!("Database already exists, verify mode");
                    is_verify = true;
                    db
                }
                Ok(None) => db_env.create_database(&mut wtxn, Some(ORDER_DATABASE_NAME))?,
                Err(e) => return Err(e.into()), // Proper error propagation
            };
        wtxn.commit()?;

        let mut wtxn = db_env.write_txn()?;

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 3 {
                return Err(anyhow!("invalid line: {}", line));
            }
            let tx_order = parts[0].parse::<u64>()?;
            let tx_hash = H256::from_str(parts[1])?;
            let block_number = parts[2].parse::<u128>()?;
            let tx_position = TxPosition {
                tx_hash,
                block_number,
            };

            if is_verify {
                let rtxn = db_env.read_txn()?;
                let ret = db.get(&rtxn, &tx_order)?;
                let ret = ret.ok_or(anyhow!("tx_order not found: {}", tx_order))?;
                rtxn.commit()?;
                assert_eq!(ret, tx_position);
            } else {
                db.put(&mut wtxn, &tx_order, &tx_position)?;
            }

            last_tx_order = tx_order;
            last_tx_hash = tx_hash;
            last_block_number = block_number;
        }

        wtxn.commit()?;

        if last_tx_order != 0 {
            let rtxn = db_env.read_txn()?;
            let ret = db.last(&rtxn)?;
            assert_eq!(
                ret,
                Some((
                    last_tx_order,
                    TxPosition {
                        tx_hash: last_tx_hash,
                        block_number: last_block_number,
                    }
                ))
            );
        }

        {
            let rtxn = db_env.read_txn()?;
            let final_count = db.iter(&rtxn)?.count();
            info!("Final record count: {}", final_count);
            rtxn.commit()?;
        }

        db_env.force_sync()?;

        Ok(())
    }

    pub fn new(db_path: PathBuf, reset_from: Option<u64>) -> anyhow::Result<Self> {
        let db_env = Self::create_env(db_path)?;
        let mut txn = db_env.write_txn()?;
        let db: Database<U64<BigEndian>, SerdeBincode<TxPosition>> =
            db_env.create_database(&mut txn, Some(ORDER_DATABASE_NAME))?;
        txn.commit()?;

        let mut indexer = Indexer {
            db_env,
            db,
            last_tx_order: 0,
            last_block_number: 0,
        };
        if let Some(from) = reset_from {
            indexer.reset_from(from)?;
        }

        indexer.init_cursor()?;
        Ok(indexer)
    }

    pub fn get_tx_position(&self, tx_order: u64) -> anyhow::Result<Option<TxPosition>> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        let ret = db.get(&rtxn, &tx_order)?;
        rtxn.commit()?;
        Ok(ret)
    }

    fn create_env(db_path: PathBuf) -> anyhow::Result<Env> {
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(MAP_SIZE) // 16G
                .max_dbs(MAX_DBS)
                .open(db_path)?
        };
        Ok(env)
    }

    // init cursor by search last tx_order
    fn init_cursor(&mut self) -> anyhow::Result<()> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        if let Some((k, v)) = db.last(&rtxn)? {
            self.last_tx_order = k;
            self.last_block_number = v.block_number;
        }
        rtxn.commit()?;
        Ok(())
    }

    fn reset_from(&self, from: u64) -> anyhow::Result<()> {
        let mut wtxn = self.db_env.write_txn()?;
        let db = self.db;

        let range = from..;
        let deleted_count = db.delete_range(&mut wtxn, &range)?;
        wtxn.commit()?;
        info!("deleted {} records from tx_order: {}", deleted_count, from);
        Ok(())
    }

    pub fn get_stats(&self) -> anyhow::Result<IndexerStats> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        let count = db.iter(&rtxn)?.count();
        rtxn.commit()?;
        Ok(IndexerStats {
            total_tx_count: count as u64,
            last_tx_order: self.last_tx_order,
            last_block_number: self.last_block_number,
        })
    }
}
