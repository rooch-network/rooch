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
use reqwest::StatusCode;
use rooch_config::RoochOpt;
use rooch_db::RoochDB;
use rooch_rpc_client::Client;
use rooch_store::RoochStore;
use rooch_types::da::chunk::chunk_from_segments;
use rooch_types::da::segment::{segment_from_bytes, SegmentID};
use rooch_types::rooch_network::RoochChainID;
use rooch_types::sequencer::SequencerInfo;
use rooch_types::transaction::{LedgerTransaction, TransactionSequenceInfo};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, RwLock};
use tokio::time;
use tracing::{error, info, warn};

pub mod exec;
pub mod index;
pub mod namespace;
pub mod pack;
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

pub(crate) fn collect_chunk(segment_dir: PathBuf, chunk_id: u128) -> anyhow::Result<Vec<u64>> {
    let mut segments = Vec::new();
    for segment_number in 0.. {
        let segment_id = SegmentID {
            chunk_id,
            segment_number,
        };
        let segment_path = segment_dir.join(segment_id.to_string());
        if !segment_path.exists() {
            if segment_number == 0 {
                return Err(anyhow::anyhow!("No segment found in chunk: {}", chunk_id));
            } else {
                break;
            }
        }

        segments.push(segment_number);
    }
    Ok(segments)
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

    let origin_chunk_count = chunks.len();
    // remove chunks that don't have segment_number 0
    // because we need to start from segment_number 0 to unpack the chunk.
    // in the download process, we download segments to tmp dir first,
    // then move them to segment dir,
    // a segment with segment_number 0 is the last segment to move,
    // so if it exists, the chunk is complete.
    let chunks: HashMap<u128, Vec<u64>> =
        chunks.into_iter().filter(|(_, v)| v.contains(&0)).collect();
    let chunk_count = chunks.len();
    if chunk_count < origin_chunk_count {
        error!(
            "Removed {} incomplete chunks, {} chunks left. Please check the segment dir: {:?} and download the missing segments.",
            origin_chunk_count - chunk_count,
            chunk_count,
            segment_dir
        );
        return Err(anyhow::anyhow!("Incomplete chunks found"));
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
    verify_order: bool,
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
    batch.verify(verify_order)?;
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

pub(crate) struct SegmentDownloader {
    open_da_path: String,
    segment_dir: PathBuf,
    next_chunk_id: u128,
    chunks: Arc<RwLock<HashMap<u128, Vec<u64>>>>,
}

impl SegmentDownloader {
    pub(crate) fn new(
        open_da_path: String,
        segment_dir: PathBuf,
        next_chunk_id: u128,
        chunks: Arc<RwLock<HashMap<u128, Vec<u64>>>>,
    ) -> anyhow::Result<Self> {
        Ok(SegmentDownloader {
            open_da_path,
            segment_dir,
            next_chunk_id,
            chunks,
        })
    }

    async fn download_chunk(
        open_da_path: String,
        segment_dir: PathBuf,
        segment_tmp_dir: PathBuf,
        chunk_id: u128,
    ) -> anyhow::Result<Option<Vec<u64>>> {
        let tmp_dir = segment_tmp_dir;
        let mut done_segments = Vec::new();
        for segment_number in 0.. {
            let segment_url = format!("{}/{}_{}", open_da_path, chunk_id, segment_number);
            let res = reqwest::get(segment_url).await?;
            if res.status().is_success() {
                let segment_bytes = res.bytes().await?;
                let segment_path = tmp_dir.join(format!("{}_{}", chunk_id, segment_number));
                let mut file = File::create(&segment_path)?;
                file.write_all(&segment_bytes)?;
                done_segments.push(segment_number);
            } else {
                if res.status() == StatusCode::NOT_FOUND {
                    if segment_number == 0 {
                        return Ok(None);
                    } else {
                        break; // no more segments for this chunk
                    }
                }
                return Err(anyhow!(
                    "Failed to download segment: {}_{}: {} ",
                    chunk_id,
                    segment_number,
                    res.status(),
                ));
            }
        }

        for segment_number in done_segments.clone().into_iter().rev() {
            let tmp_path = tmp_dir.join(format!("{}_{}", chunk_id, segment_number));
            let dst_path = segment_dir.join(format!("{}_{}", chunk_id, segment_number));
            fs::rename(tmp_path, dst_path)?;
        }

        Ok(Some(done_segments))
    }

    pub(crate) fn run_in_background(
        self,
        shutdown_signal: watch::Receiver<()>,
    ) -> anyhow::Result<()> {
        let base_url = self.open_da_path;
        let segment_dir = self.segment_dir;
        let tmp_dir = segment_dir.join("tmp");
        fs::create_dir_all(&tmp_dir)?;
        let next_chunk_id = self.next_chunk_id;

        tokio::spawn(async move {
            let mut shutdown_signal = shutdown_signal;

            let mut interval = time::interval(Duration::from_secs(60 * 5));
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

            let mut chunk_id = next_chunk_id;
            let base_url = base_url.clone();

            loop {
                tokio::select! {
                    _ = shutdown_signal.changed() => {
                        info!("Shutting down segments download task.");
                        break;
                    }
                    _ = interval.tick() => {
                        loop {
                            let res = Self::download_chunk(base_url.clone(), segment_dir.clone(), tmp_dir.clone(), chunk_id).await;
                            match res {
                                Ok(Some(segments)) => {
                                    let mut chunks = self.chunks.write().await;
                                    chunks.insert(chunk_id, segments);
                                    chunk_id += 1;
                                }
                                Err(e) => {
                                    warn!("Failed to download chunk: {}, error: {}", chunk_id, e);
                                    break;
                                }
                            _ => {
                                break;
                                }}
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

pub(crate) struct LedgerTxGetter {
    segment_dir: PathBuf,
    chunks: Arc<RwLock<HashMap<u128, Vec<u64>>>>,
    client: Option<Client>,
    exp_roots: Arc<RwLock<HashMap<u64, (H256, H256)>>>,
    max_chunk_id: u128,
}

impl LedgerTxGetter {
    pub(crate) fn new(segment_dir: PathBuf) -> anyhow::Result<Self> {
        let (chunks, _min_chunk_id, max_chunk_id) = collect_chunks(segment_dir.clone())?;

        Ok(LedgerTxGetter {
            segment_dir,
            chunks: Arc::new(RwLock::new(chunks)),
            client: None,
            exp_roots: Arc::new(RwLock::new(HashMap::new())),
            max_chunk_id,
        })
    }

    pub(crate) fn new_with_auto_sync(
        open_da_path: String,
        segment_dir: PathBuf,
        client: Client,
        exp_roots: Arc<RwLock<HashMap<u64, (H256, H256)>>>,
        shutdown_signal: watch::Receiver<()>,
    ) -> anyhow::Result<Self> {
        let (chunks, _min_chunk_id, max_chunk_id) = collect_chunks(segment_dir.clone())?;

        let chunks_to_sync = Arc::new(RwLock::new(chunks.clone()));

        let downloader = SegmentDownloader::new(
            open_da_path,
            segment_dir.clone(),
            max_chunk_id + 1,
            chunks_to_sync.clone(),
        )?;
        downloader.run_in_background(shutdown_signal)?;
        Ok(LedgerTxGetter {
            segment_dir,
            chunks: chunks_to_sync,
            client: Some(client),
            exp_roots,
            max_chunk_id,
        })
    }

    pub(crate) async fn load_ledger_tx_list(
        &self,
        chunk_id: u128,
        must_has: bool,
    ) -> anyhow::Result<Option<Vec<LedgerTransaction>>> {
        let tx_list_opt = self
            .chunks
            .read()
            .await
            .get(&chunk_id)
            .cloned()
            .map_or_else(
                || {
                    if must_has {
                        Err(anyhow::anyhow!("No segment found in chunk {}", chunk_id))
                    } else {
                        Ok(None)
                    }
                },
                |segment_numbers| {
                    let tx_list = get_tx_list_from_chunk(
                        self.segment_dir.clone(),
                        chunk_id,
                        segment_numbers.clone(),
                        true,
                    )?;
                    Ok(Some(tx_list))
                },
            )?;
        if let Some(tx_list) = tx_list_opt {
            if let Some(client) = &self.client {
                let exp_roots = self.exp_roots.clone();
                let mut last_tx = tx_list.last().unwrap().clone();
                let tx_order = last_tx.sequence_info.tx_order;
                let last_tx_hash = last_tx.tx_hash();
                let resp = client
                    .rooch
                    .get_transactions_by_hash(vec![last_tx_hash])
                    .await?;
                let tx_info = resp.into_iter().next().flatten().ok_or_else(|| {
                    anyhow!("No transaction info found for tx: {:?}", last_tx_hash)
                })?;
                let tx_order_in_resp = tx_info.transaction.sequence_info.tx_order.0;
                if tx_order_in_resp != tx_order {
                    return Err(anyhow!(
                        "failed to request tx by RPC: Tx order mismatch, expect: {}, actual: {}",
                        tx_order,
                        tx_order_in_resp
                    ));
                } else {
                    let execution_info_opt = tx_info.execution_info;
                    // not all sequenced tx could be executed successfully
                    if let Some(execution_info) = execution_info_opt {
                        let tx_state_root = execution_info.state_root.0;
                        let tx_accumulator_root =
                            tx_info.transaction.sequence_info.tx_accumulator_root.0;
                        let mut exp_roots = exp_roots.write().await;
                        exp_roots.insert(tx_order, (tx_state_root, tx_accumulator_root));
                    }
                }
            }
            Ok(Some(tx_list))
        } else {
            Ok(None)
        }
    }

    // only valid for no segments sync
    pub(crate) fn get_max_chunk_id(&self) -> u128 {
        self.max_chunk_id
    }
}

pub(crate) struct TxMetaStore {
    tx_position_indexer: TxPositionIndexer,
    exp_roots: Arc<RwLock<HashMap<u64, (H256, H256)>>>, // tx_order -> (state_root, accumulator_root)
    max_verified_tx_order: u64,
    transaction_store: TransactionDBStore,
    rooch_store: RoochStore,
}

struct ExpRootsMap {
    exp_roots: HashMap<u64, (H256, H256)>,
    max_verified_tx_order: u64,
}

impl TxMetaStore {
    pub(crate) async fn new(
        tx_position_indexer_path: PathBuf,
        exp_roots_path: PathBuf,
        segment_dir: PathBuf,
        transaction_store: TransactionDBStore,
        rooch_store: RoochStore,
        max_block_number: Option<u128>,
    ) -> anyhow::Result<Self> {
        let tx_position_indexer = TxPositionIndexer::new_with_updates(
            tx_position_indexer_path,
            None,
            Some(segment_dir),
            max_block_number,
        )
        .await?;
        let exp_roots_map = Self::load_exp_roots(exp_roots_path)?;
        let max_verified_tx_order = exp_roots_map.max_verified_tx_order;
        Ok(TxMetaStore {
            tx_position_indexer,
            exp_roots: Arc::new(RwLock::new(exp_roots_map.exp_roots)),
            max_verified_tx_order,
            transaction_store,
            rooch_store,
        })
    }

    pub(crate) fn get_exp_roots_map(&self) -> Arc<RwLock<HashMap<u64, (H256, H256)>>> {
        self.exp_roots.clone()
    }

    fn load_exp_roots(exp_roots_path: PathBuf) -> anyhow::Result<ExpRootsMap> {
        let mut exp_roots = HashMap::new();
        let mut max_verified_tx_order = 0;

        let mut reader = BufReader::new(File::open(exp_roots_path)?);
        for line in reader.by_ref().lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(':').collect();
            let tx_order = parts[0].parse::<u64>()?;
            let state_root_raw = parts[1];
            let state_root = if state_root_raw == "null" {
                H256::zero()
            } else {
                H256::from_str(state_root_raw)?
            };
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

    pub(crate) async fn get_exp_roots(&self, tx_order: u64) -> Option<(H256, H256)> {
        self.exp_roots.read().await.get(&tx_order).cloned()
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
        let file = File::create(file_path)?;
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone()?);
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
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

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

    pub(crate) async fn new_with_updates(
        db_path: PathBuf,
        reset_from: Option<u64>,
        segment_dir: Option<PathBuf>,
        max_block_number: Option<u128>,
    ) -> anyhow::Result<Self> {
        let mut indexer = TxPositionIndexer::new(db_path, reset_from)?;
        let stats_before_reset = indexer.get_stats()?;
        info!("indexer stats after load: {:?}", stats_before_reset);
        indexer
            .updates_by_segments(segment_dir, max_block_number)
            .await?;
        info!("indexer stats after updates: {:?}", indexer.get_stats()?);
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

    pub(crate) async fn updates_by_segments(
        &mut self,
        segment_dir: Option<PathBuf>,
        max_block_number: Option<u128>,
    ) -> anyhow::Result<()> {
        let segment_dir = segment_dir.ok_or_else(|| anyhow!("segment_dir is required"))?;
        let ledger_tx_loader = LedgerTxGetter::new(segment_dir)?;
        let stop_at = if let Some(max_block_number) = max_block_number {
            min(max_block_number, ledger_tx_loader.get_max_chunk_id())
        } else {
            ledger_tx_loader.get_max_chunk_id()
        };
        let mut block_number = self.last_block_number; // avoiding partial indexing
        let mut expected_tx_order = self.last_tx_order + 1;
        let mut done_block = 0;

        while block_number <= stop_at {
            let tx_list = ledger_tx_loader
                .load_ledger_tx_list(block_number, true)
                .await?;
            let tx_list = tx_list.unwrap();
            {
                let db = self.db;
                let mut wtxn = self.db_env.write_txn()?;
                for mut ledger_tx in tx_list {
                    let tx_order = ledger_tx.sequence_info.tx_order;
                    if tx_order < expected_tx_order {
                        continue;
                    }
                    if tx_order == self.last_tx_order + 1 {
                        info!(
                            "begin to index block: {}, tx_order: {}",
                            block_number, tx_order
                        );
                    }
                    if tx_order != expected_tx_order {
                        return Err(anyhow!(
                            "tx_order not continuous, expect: {}, got: {}",
                            expected_tx_order,
                            tx_order
                        ));
                    }
                    let tx_hash = ledger_tx.tx_hash();
                    let tx_position = TxPosition {
                        tx_order,
                        tx_hash,
                        block_number,
                    };
                    db.put(&mut wtxn, &tx_order, &tx_position)?;
                    expected_tx_order += 1;
                }
                wtxn.commit()?;
            }
            block_number += 1;
            done_block += 1;
            if done_block % 1000 == 0 {
                info!(
                    "done: block_cnt: {}; next_block_number: {}",
                    done_block, block_number
                );
            }
        }

        self.init_cursor()
    }

    pub(crate) fn close(&self) -> anyhow::Result<()> {
        let env = self.db_env.clone();
        env.force_sync()?;
        drop(env);
        Ok(())
    }
}
