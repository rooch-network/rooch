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

pub(crate) struct LedgerTxGetter {
    segment_dir: PathBuf,
    chunks: HashMap<u128, Vec<u64>>,
    max_chunk_id: u128,
}

impl LedgerTxGetter {
    pub(crate) fn new(segment_dir: PathBuf) -> anyhow::Result<Self> {
        let (chunks, _min_chunk_id, max_chunk_id) = collect_chunks(segment_dir.clone())?;

        Ok(LedgerTxGetter {
            segment_dir,
            chunks,
            max_chunk_id,
        })
    }

    pub(crate) fn load_ledger_tx_list(
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

    pub(crate) fn get_max_chunk_id(&self) -> u128 {
        self.max_chunk_id
    }
}

pub(crate) struct TxMetaStore {
    tx_position_indexer: TxPositionIndexer,
    exp_roots: HashMap<u64, (H256, H256)>, // tx_order -> (state_root, accumulator_root)
    max_verified_tx_order: u64,
    transaction_store: TransactionDBStore,
    rooch_store: RoochStore,
}

struct ExpRootsMap {
    exp_roots: HashMap<u64, (H256, H256)>,
    max_verified_tx_order: u64,
}

impl TxMetaStore {
    pub(crate) fn new(
        tx_position_indexer_path: PathBuf,
        exp_roots_path: PathBuf,
        transaction_store: TransactionDBStore,
        rooch_store: RoochStore,
    ) -> anyhow::Result<Self> {
        let tx_position_indexer = TxPositionIndexer::new(tx_position_indexer_path, None)?;
        let exp_roots_map = Self::load_exp_roots(exp_roots_path)?;
        Ok(TxMetaStore {
            tx_position_indexer,
            exp_roots: exp_roots_map.exp_roots,
            max_verified_tx_order: exp_roots_map.max_verified_tx_order,
            transaction_store,
            rooch_store,
        })
    }

    fn load_exp_roots(exp_roots_path: PathBuf) -> anyhow::Result<ExpRootsMap> {
        let mut exp_roots = HashMap::new();
        let mut max_verified_tx_order = 0;

        let mut reader = BufReader::new(File::open(exp_roots_path)?);
        for line in reader.by_ref().lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split(':').collect();
            let tx_order = parts[0].parse::<u64>()?;
            let state_root = H256::from_str(parts[1])?;
            let accumulator_root = H256::from_str(parts[2])?;
            exp_roots.insert(tx_order, (state_root, accumulator_root));
            if tx_order > max_verified_tx_order {
                max_verified_tx_order = tx_order;
            }
        }
        Ok(ExpRootsMap {
            exp_roots,
            max_verified_tx_order,
        })
    }

    pub(crate) fn get_exp_roots(&self, tx_order: u64) -> Option<(H256, H256)> {
        self.exp_roots.get(&tx_order).cloned()
    }

    pub(crate) fn get_max_verified_tx_order(&self) -> u64 {
        self.max_verified_tx_order
    }

    pub(crate) fn get_tx_hash(&self, tx_order: u64) -> Option<H256> {
        let r = self
            .tx_position_indexer
            .get_tx_position(tx_order)
            .ok()
            .flatten();
        r.map(|tx_position| tx_position.tx_hash)
    }

    pub(crate) fn get_tx_positions_in_range(
        &self,
        start_tx_order: u64,
        end_tx_order: u64,
    ) -> anyhow::Result<Vec<TxPosition>> {
        self.tx_position_indexer
            .get_tx_positions_in_range(start_tx_order, end_tx_order)
    }

    pub(crate) fn find_last_executed(&self) -> anyhow::Result<Option<TxPosition>> {
        let predicate = |tx_order: &u64| self.has_executed_by_tx_order(*tx_order);
        let last_tx_order = self.tx_position_indexer.last_tx_order;
        if last_tx_order == 0 {
            // no tx indexed through DA segments
            return Ok(None);
        }
        if !predicate(&1) {
            return Ok(None); // first tx in DA segments is not executed
        }
        if predicate(&last_tx_order) {
            return self.tx_position_indexer.get_tx_position(last_tx_order); // last tx is executed
        }

        // binary search [1, self.tx_position_indexer.last_tx_order]
        let mut left = 1; // first tx is executed, has checked
        let mut right = last_tx_order;

        while left + 1 < right {
            let mid = left + (right - left) / 2;
            if predicate(&mid) {
                left = mid; // mid is true, the final answer is mid or on the right
            } else {
                right = mid; // mid is false, the final answer is on the left
            }
        }

        // left is the last true position
        self.tx_position_indexer.get_tx_position(left)
    }

    pub(crate) fn find_tx_block(&self, tx_order: u64) -> Option<u128> {
        let r = self
            .tx_position_indexer
            .get_tx_position(tx_order)
            .ok()
            .flatten();
        r.map(|tx_position| tx_position.block_number)
    }

    fn has_executed_by_tx_order(&self, tx_order: u64) -> bool {
        self.get_tx_hash(tx_order)
            .map_or(false, |tx_hash| self.has_executed(tx_hash))
    }

    fn has_executed(&self, tx_hash: H256) -> bool {
        self.get_execution_info(tx_hash)
            .map_or(false, |info| info.is_some())
    }

    pub(crate) fn get_execution_info(
        &self,
        tx_hash: H256,
    ) -> anyhow::Result<Option<TransactionExecutionInfo>> {
        self.transaction_store.get_tx_execution_info(tx_hash)
    }

    pub(crate) fn get_sequencer_info(
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
pub(crate) struct TxPosition {
    pub(crate) tx_order: u64,
    pub(crate) tx_hash: H256,
    pub(crate) block_number: u128,
}

pub(crate) struct TxPositionIndexer {
    db_env: Env,
    db: Database<U64<BigEndian>, SerdeBincode<TxPosition>>,
    last_tx_order: u64,
    last_block_number: u128,
}

#[derive(Debug, Serialize)]
pub(crate) struct TxPositionIndexerStats {
    pub(crate) total_tx_count: u64,
    pub(crate) last_tx_order: u64,
    pub(crate) last_block_number: u128,
}

impl TxPositionIndexer {
    pub(crate) fn load_or_dump(
        db_path: PathBuf,
        file_path: PathBuf,
        dump: bool,
    ) -> anyhow::Result<()> {
        if dump {
            let indexer = TxPositionIndexer::new(db_path, None)?;
            indexer.dump_to_file(file_path)
        } else {
            TxPositionIndexer::load_from_file(db_path, file_path)
        }
    }

    pub(crate) fn dump_to_file(&self, file_path: PathBuf) -> anyhow::Result<()> {
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

    pub(crate) fn load_from_file(db_path: PathBuf, file_path: PathBuf) -> anyhow::Result<()> {
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
                tx_order,
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
                        tx_order: last_tx_order,
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

    pub(crate) fn new(db_path: PathBuf, reset_from: Option<u64>) -> anyhow::Result<Self> {
        let db_env = Self::create_env(db_path)?;
        let mut txn = db_env.write_txn()?;
        let db: Database<U64<BigEndian>, SerdeBincode<TxPosition>> =
            db_env.create_database(&mut txn, Some(ORDER_DATABASE_NAME))?;
        txn.commit()?;

        let mut indexer = TxPositionIndexer {
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

    pub(crate) fn get_tx_position(&self, tx_order: u64) -> anyhow::Result<Option<TxPosition>> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        let ret = db.get(&rtxn, &tx_order)?;
        rtxn.commit()?;
        Ok(ret)
    }

    pub(crate) fn get_tx_positions_in_range(
        &self,
        start: u64,
        end: u64,
    ) -> anyhow::Result<Vec<TxPosition>> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        let mut tx_positions = Vec::new();
        let range = start..=end;
        let mut iter = db.range(&rtxn, &range)?;
        while let Some((_k, v)) = iter.next().transpose()? {
            tx_positions.push(v);
        }
        drop(iter);
        rtxn.commit()?;
        Ok(tx_positions)
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
    pub(crate) fn init_cursor(&mut self) -> anyhow::Result<()> {
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

    pub(crate) fn get_stats(&self) -> anyhow::Result<TxPositionIndexerStats> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        let count = db.iter(&rtxn)?.count();
        rtxn.commit()?;
        Ok(TxPositionIndexerStats {
            total_tx_count: count as u64,
            last_tx_order: self.last_tx_order,
            last_block_number: self.last_block_number,
        })
    }

    pub(crate) fn close(&self) -> anyhow::Result<()> {
        let env = self.db_env.clone();
        env.force_sync()?;
        drop(env);
        Ok(())
    }
}
