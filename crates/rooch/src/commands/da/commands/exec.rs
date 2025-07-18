// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::da::commands::{
    build_rooch_db, LedgerTxGetter, SequencedTxStore, StateRootFetcher, TxMetaStore,
};
use crate::utils::derive_builtin_genesis_namespace;
use anyhow::Context;
use bitcoin::hashes::Hash;
use bitcoin_client::actor::client::BitcoinClientConfig;
use bitcoin_client::proxy::BitcoinClientProxy;
use clap::Parser;
use coerce::actor::system::ActorSystem;
use coerce::actor::IntoActor;
use hdrhistogram::Histogram;
use metrics::RegistryService;
use moveos_common::utils::to_bytes;
use moveos_eventbus::bus::EventBus;
use moveos_store::config_store::STARTUP_INFO_KEY;
use moveos_store::{MoveOSStore, CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME};
use moveos_types::startup_info;
use moveos_types::transaction::{TransactionExecutionInfo, VerifiedMoveOSTransaction};
use raw_store::rocks::batch::WriteBatch;
use raw_store::traits::DBStore;
use rooch_anomalies::{load_tx_anomalies, TxAnomalies};
use rooch_common::humanize::parse_bytes;
use rooch_config::R_OPT_NET_HELP;
use rooch_db::RoochDB;
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_notify::actor::NotifyActor;
use rooch_notify::subscription_handler::SubscriptionHandler;
use rooch_pipeline_processor::actor::processor::is_vm_panic_error;
use rooch_store::meta_store::SEQUENCER_INFO_KEY;
use rooch_store::META_SEQUENCER_INFO_COLUMN_FAMILY_NAME;
use rooch_types::bitcoin::types::Block as BitcoinBlock;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID};
use rooch_types::sequencer::SequencerInfo;
use rooch_types::transaction::{
    L1BlockWithBody, LedgerTransaction, LedgerTxData, TransactionSequenceInfo,
};
use std::cmp::{max, min, PartialEq};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio::time;
use tokio::time::sleep;
use tracing::{info, warn};

/// exec LedgerTransaction List for verification.
#[derive(Debug, Parser)]
pub struct ExecCommand {
    #[clap(
        long = "mode",
        default_value = "sync",
        help = "Execution mode: exec, seq, all, sync, sync-exec. Default is sync"
    )]
    pub mode: ExecMode,
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(
        long = "tx-position",
        help = "Path to tx_order:tx_hash:l2_block_number database directory"
    )]
    pub tx_position_path: PathBuf,
    #[clap(
        long = "exp-root",
        help = "Path to tx_order:state_root:accumulator_root file(results from RoochNetwork), for caching expected roots avoiding blocking on RPC requests"
    )]
    pub exp_root_path: PathBuf,
    #[clap(
        long = "rollback",
        help = "rollback to tx order. If not set or ge executed_tx_order, start from executed_tx_order+1(nothing to do); otherwise, rollback to this order."
    )]
    pub rollback: Option<u64>,
    #[clap(
        long = "open-da",
        help = "open da path for downloading chunks from DA. Working with `mode=sync`"
    )]
    pub open_da_path: Option<String>,

    #[clap(
        long = "force-align",
        help = "force align to min(last_sequenced_tx_order, last_executed_tx_order)"
    )]
    pub force_align: bool,
    #[clap(long = "max-block-number", help = "Max block number to exec")]
    pub max_block_number: Option<u128>,
    #[clap(long = "bypass-verify", help = "bypass verification of state root")]
    pub bypass_verify: bool,

    #[clap(long = "btc-rpc-url")]
    pub btc_rpc_url: String,
    #[clap(long = "btc-rpc-user-name")]
    pub btc_rpc_user_name: String,
    #[clap(long = "btc-rpc-password")]
    pub btc_rpc_password: String,
    #[clap(long = "btc-local-block-store-dir")]
    pub btc_local_block_store_dir: Option<PathBuf>,

    #[clap(
        name = "rocksdb-row-cache-size",
        long,
        help = "rocksdb row cache size, default 128M"
    )]
    pub row_cache_size: Option<String>,
    #[clap(
        name = "rocksdb-block-cache-size",
        long,
        help = "rocksdb block cache size, default 4G"
    )]
    pub block_cache_size: Option<String>,
    #[clap(long = "enable-rocks-stats", help = "rocksdb-enable-statistics")]
    pub enable_rocks_stats: bool,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: PathBuf,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: BuiltinChainID,
    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ExecMode {
    /// Only execute transactions, no sequence updates
    Exec,
    /// Only update sequence data, no execution
    Seq,
    /// Execute transactions and update sequence data
    All,
    /// Sync from DA automatically and `All` mode
    Sync,
    /// Sync from DA automatically and `Exec` mode
    SyncExec,
}

impl PartialEq for ExecMode {
    fn eq(&self, other: &Self) -> bool {
        self.as_bits() == other.as_bits()
    }
}

impl ExecMode {
    pub fn as_bits(&self) -> u8 {
        match self {
            ExecMode::Exec => 0b10,
            ExecMode::Seq => 0b01,
            ExecMode::All => 0b11,
            ExecMode::Sync => 0b111,
            ExecMode::SyncExec => 0b110,
        }
    }

    pub fn need_sync(&self) -> bool {
        self.as_bits() & 0b100 != 0
    }

    pub fn need_exec(&self) -> bool {
        self.as_bits() & 0b10 != 0
    }

    pub fn need_seq(&self) -> bool {
        self.as_bits() & 0b01 != 0
    }

    pub fn need_all(&self) -> bool {
        self.as_bits() & 0b11 == 0b11
    }

    pub fn get_verify_targets_str(&self, bypass_verify: bool) -> Option<String> {
        let raw_targets = match self {
            ExecMode::Exec => "state root",
            ExecMode::Seq => "accumulator root",
            ExecMode::All => "state+accumulator root",
            ExecMode::Sync => "state+accumulator root",
            ExecMode::SyncExec => "state root",
        };
        if bypass_verify {
            if raw_targets == "state root" {
                return None;
            }
            if raw_targets == "state+accumulator root" {
                return Some("accumulator root".to_string());
            }
        }
        Some(raw_targets.to_string())
    }
}

impl ExecCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (shutdown_tx, shutdown_rx) = watch::channel(());

        let shutdown_tx_clone = shutdown_tx.clone();

        tokio::spawn(async move {
            #[cfg(unix)]
            let shutdown_signal = async {
                let mut sigterm = signal(SignalKind::terminate()).unwrap();
                let mut sigint = signal(SignalKind::interrupt()).unwrap();
                tokio::select! {
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM signal");
                    },
                    _ = sigint.recv() => {
                        info!("Received SIGINT signal");
                    },
                }
            };

            #[cfg(not(unix))]
            let shutdown_signal = async {
                ctrl_c().await.expect("Failed to listen for Ctrl+C signal");
            };

            shutdown_signal.await;
            shutdown_tx_clone.send(()).unwrap();
        });

        let exec_inner = self.build_exec_inner(shutdown_rx.clone()).await?;
        exec_inner.run(shutdown_rx).await?;

        let _ = shutdown_tx.send(());

        Ok(())
    }

    async fn build_exec_inner(
        &self,
        shutdown_signal: watch::Receiver<()>,
    ) -> anyhow::Result<ExecInner> {
        let actor_system = ActorSystem::global_system();

        let row_cache_size = self
            .row_cache_size
            .clone()
            .and_then(|v| parse_bytes(&v).ok());
        let block_cache_size = self
            .block_cache_size
            .clone()
            .and_then(|v| parse_bytes(&v).ok());

        let (executor, moveos_store, rooch_db) = build_executor_and_store(
            Some(self.base_data_dir.clone()),
            Some(RoochChainID::Builtin(self.chain_id)),
            &actor_system,
            self.enable_rocks_stats,
            row_cache_size,
            block_cache_size,
        )
        .await?;

        let genesis_namespace = derive_builtin_genesis_namespace(self.chain_id)?;
        let tx_anomalies = load_tx_anomalies(genesis_namespace.clone())?;

        let check_l1_tx_executed_start_from = tx_anomalies
            .as_ref()
            .and_then(|anomalies| anomalies.check_l1_tx_executed_start_from)
            .unwrap_or(0);

        let sequenced_tx_store =
            SequencedTxStore::new(rooch_db.rooch_store.clone(), tx_anomalies.clone())?;

        let bitcoin_client_proxy = build_btc_client_proxy(
            self.btc_rpc_url.clone(),
            self.btc_rpc_user_name.clone(),
            self.btc_rpc_password.clone(),
            self.btc_local_block_store_dir.clone(),
            &actor_system,
        )
        .await?;

        let tx_meta_store = TxMetaStore::new(
            self.tx_position_path.clone(),
            self.exp_root_path.clone(),
            self.segment_dir.clone(),
            moveos_store.transaction_store,
            rooch_db.rooch_store.clone(),
            self.max_block_number,
        )
        .await?;

        let exp_roots = tx_meta_store.get_exp_roots();
        let client = self.context_options.build()?.get_client().await?;
        let state_root_fetcher =
            StateRootFetcher::new(client, exp_roots.clone(), tx_anomalies.clone());

        let ledger_tx_loader = if self.mode.need_sync() {
            LedgerTxGetter::new_with_auto_sync(
                self.open_da_path.clone().unwrap(),
                self.segment_dir.clone(),
                shutdown_signal,
            )?
        } else {
            LedgerTxGetter::new(self.segment_dir.clone(), false)?
        };
        info!(
            "auto sync ledger tx getter is: {} with mode: {:?}",
            self.mode.need_sync(),
            self.mode
        );
        Ok(ExecInner {
            mode: self.mode,
            bypass_verify: self.bypass_verify,
            force_align: self.force_align,
            ledger_tx_getter: ledger_tx_loader,
            tx_meta_store,
            sequenced_tx_store,
            bitcoin_client_proxy,
            executor,
            produced: Arc::new(AtomicU64::new(0)),
            done: Arc::new(AtomicU64::new(0)),
            executed_tx_order: Arc::new(AtomicU64::new(0)),
            rollback: self.rollback,
            rooch_db,
            tx_anomalies,
            state_root_fetcher,
            check_l1_tx_executed_start_from,
        })
    }
}

struct ExecInner {
    mode: ExecMode,
    bypass_verify: bool,
    force_align: bool,

    ledger_tx_getter: LedgerTxGetter,
    tx_meta_store: TxMetaStore,

    sequenced_tx_store: SequencedTxStore,

    bitcoin_client_proxy: BitcoinClientProxy,
    executor: ExecutorProxy,

    rooch_db: RoochDB,
    rollback: Option<u64>,

    produced: Arc<AtomicU64>,
    done: Arc<AtomicU64>,
    executed_tx_order: Arc<AtomicU64>,

    state_root_fetcher: StateRootFetcher,

    tx_anomalies: Option<TxAnomalies>,
    check_l1_tx_executed_start_from: u64,
}

struct ExecMsg {
    tx_order: u64,
    ledger_tx: LedgerTransaction,
    l1_block_with_body: Option<L1BlockWithBody>,
}

impl ExecInner {
    fn start_logging_task(&self, shutdown_signal: watch::Receiver<()>) {
        let done_cloned = self.done.clone();
        let executed_tx_order_cloned = self.executed_tx_order.clone();
        let produced_cloned = self.produced.clone();

        tokio::spawn(async move {
            let mut shutdown_signal = shutdown_signal;

            let mut interval = time::interval(Duration::from_secs(60));
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = shutdown_signal.changed() => {
                        info!("Shutting down logging task.");
                        break;
                    }
                    _ = interval.tick() => {
                        let done = done_cloned.load(std::sync::atomic::Ordering::Relaxed);
                        let executed_tx_order = executed_tx_order_cloned.load(std::sync::atomic::Ordering::Relaxed);
                        let produced = produced_cloned.load(std::sync::atomic::Ordering::Relaxed);

                        info!(
                            "produced: {}, done: {}, max executed_tx_order: {}",
                            produced,
                            done,
                            executed_tx_order
                        );
                    }
                }
            }
        });
    }

    // Joins the producer and consumer, handling results.
    async fn join_producer_and_consumer(
        &self,
        producer: impl std::future::Future<Output = anyhow::Result<()>>,
        consumer: impl std::future::Future<Output = anyhow::Result<()>>,
    ) -> anyhow::Result<()> {
        let (producer_result, consumer_result) = tokio::join!(producer, consumer);

        // Error handling: Match the producer and consumer results.
        match (producer_result, consumer_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(producer_err), Ok(())) => Err(producer_err.context("Error in producer")),
            (Ok(()), Err(consumer_err)) => Err(consumer_err.context("Error in consumer")),
            (Err(producer_err), Err(consumer_err)) => {
                let combined_error = producer_err.context("Error in producer");
                Err(combined_error.context(format!("Error in consumer: {:?}", consumer_err)))
            }
        }
    }

    async fn run(&self, shutdown_signal: watch::Receiver<()>) -> anyhow::Result<()> {
        self.start_logging_task(shutdown_signal.clone());

        // larger buffer size to avoid rx starving caused by consumer has to access disks and request btc block.
        // after consumer load data(ledger_tx) from disk/btc client, burst to executor, need large buffer to avoid blocking.
        // 16384 is a magic number, it's a trade-off between memory usage and performance. (usually tx count inside a block is under 8192, MAX_TXS_PER_BLOCK_IN_FIX)
        let (tx, rx) = tokio::sync::mpsc::channel(16384);
        let producer = self.produce_tx(tx, shutdown_signal);
        let consumer = self.consume_tx(rx);

        self.join_producer_and_consumer(producer, consumer).await
    }

    fn update_startup_info_after_rollback(
        &self,
        execution_info: Option<TransactionExecutionInfo>,
        sequencer_info: Option<TransactionSequenceInfo>,
    ) -> anyhow::Result<()> {
        let rollback_sequencer_info = if let Some(sequencer_info) = sequencer_info {
            Some(SequencerInfo::new(
                sequencer_info.tx_order,
                sequencer_info.tx_accumulator_info(),
            ))
        } else {
            None
        };
        let rollback_startup_info = if let Some(execution_info) = execution_info {
            Some(startup_info::StartupInfo::new(
                execution_info.state_root,
                execution_info.size,
            ))
        } else {
            None
        };

        let inner_store = &self.rooch_db.rooch_store.store_instance;
        let mut write_batch = WriteBatch::new();
        let mut cf_names = Vec::new();
        if let Some(rollback_sequencer_info) = rollback_sequencer_info {
            cf_names.push(META_SEQUENCER_INFO_COLUMN_FAMILY_NAME);
            write_batch.put(
                to_bytes(SEQUENCER_INFO_KEY).unwrap(),
                to_bytes(&rollback_sequencer_info).unwrap(),
            )?;
        }
        if let Some(rollback_startup_info) = rollback_startup_info {
            cf_names.push(CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME);
            write_batch.put(
                to_bytes(STARTUP_INFO_KEY).unwrap(),
                to_bytes(&rollback_startup_info).unwrap(),
            )?;
        }

        inner_store.write_batch_across_cfs(cf_names, write_batch, true)
    }

    async fn produce_tx(
        &self,
        tx: Sender<ExecMsg>,
        mut shutdown_signal: watch::Receiver<()>,
    ) -> anyhow::Result<()> {
        let last_executed_opt = self.tx_meta_store.find_last_executed()?;
        let last_executed_tx_order = match last_executed_opt {
            Some(v) => v.tx_order,
            None => 0,
        };
        let mut next_tx_order = last_executed_tx_order + 1;

        let last_sequenced_tx = self
            .sequenced_tx_store
            .get_last_sequenced_tx_order_in_last_job();
        let next_sequence_tx = last_sequenced_tx + 1;

        let last_full_executed_tx_order = min(last_sequenced_tx, last_executed_tx_order);
        let last_partial_executed_tx_order = max(last_sequenced_tx, last_executed_tx_order);

        let mut rollback_to = self.rollback;
        let origin_rollback = self.rollback;
        if self.mode.need_all() && next_tx_order != next_sequence_tx {
            warn! {
                "Last executed tx order: {}, last sequenced tx order: {}; run exec/seq only to catch up or run with `force-align` to rollback to tx order: {}",
                last_executed_tx_order,
                last_sequenced_tx,
                last_full_executed_tx_order
            }

            if rollback_to.is_none() {
                rollback_to = Some(last_full_executed_tx_order);
            } else {
                rollback_to = Some(min(rollback_to.unwrap(), last_full_executed_tx_order));
            }
        }
        if rollback_to != origin_rollback && !self.force_align {
            return Ok(());
        }

        // If rollback not set or ge `last_partial_executed_tx_order`: nothing to do;
        // otherwise, rollback to this order
        if let Some(rollback) = rollback_to {
            if rollback < last_partial_executed_tx_order {
                let new_last_and_rollback = self
                    .tx_meta_store
                    .get_tx_positions_in_range(rollback, last_partial_executed_tx_order)?;
                // split into two parts, the first get execution info for new startup, all others rollback
                let (new_last, rollback_part) = new_last_and_rollback.split_first().unwrap();
                info!(
                    "Start to rollback transactions tx_order: [{}, {}]",
                    rollback_part.first().unwrap().tx_order,
                    rollback_part.last().unwrap().tx_order,
                );
                for need_revert in rollback_part.iter() {
                    self.rooch_db
                        .revert_tx_unsafe(need_revert.tx_order, need_revert.tx_hash)
                        .map_err(|err| {
                            anyhow::anyhow!(
                                "Error reverting transaction {}: {:?}",
                                need_revert.tx_order,
                                err
                            )
                        })?;
                }
                let rollback_execution_info =
                    self.tx_meta_store.get_execution_info(new_last.tx_hash)?;
                let rollback_sequencer_info =
                    self.tx_meta_store.get_sequencer_info(new_last.tx_hash)?;
                self.update_startup_info_after_rollback(
                    rollback_execution_info,
                    rollback_sequencer_info,
                )?;
                info!("Rollback transactions done. Please RESTART process without rollback.");
                return Ok(()); // rollback done, need to restart to get new state_root for startup rooch store
            }
        };

        let mut next_block_number = last_executed_opt
            .map(|v| v.block_number) // next_tx_order and last executed tx may be in the same block
            .unwrap_or(0);

        if !self.mode.need_exec() {
            next_tx_order = last_sequenced_tx + 1;
            next_block_number = self.tx_meta_store.find_tx_block(next_tx_order).unwrap();
        }
        info!(
            "Start to produce transactions from tx_order: {}, check from block: {}",
            next_tx_order, next_block_number,
        );
        let mut produced_tx_order = 0;
        let mut reach_end = false;
        let max_verified_tx_order = self.tx_meta_store.get_max_verified_tx_order();
        loop {
            if reach_end {
                break;
            }

            tokio::select! {
                _ = shutdown_signal.changed() => {
                    info!("Shutting down producer task.");
                    break;
                }
                result = self
                .ledger_tx_getter
                .load_ledger_tx_list(next_block_number, false, true) => {
                let tx_list = result?;
            if tx_list.is_none() {
                if self.mode.need_sync() {
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
                next_block_number -= 1; // no chunk belongs to this block_number
                break;
            }
            let tx_list = tx_list.unwrap();
            let last_tx_order_in_list = tx_list.last().map(|tx| tx.sequence_info.tx_order).unwrap();
            self.state_root_fetcher.fetch_and_add(last_tx_order_in_list).await?;
            for ledger_tx in tx_list {
                let tx_order = ledger_tx.sequence_info.tx_order;
                if tx_order > max_verified_tx_order && !self.mode.need_sync() {
                    reach_end = true;
                    break;
                }
                if tx_order < next_tx_order {
                    continue;
                }

                let l1_block_with_body = match &ledger_tx.data {
                    LedgerTxData::L1Block(block) => {
                        let block_hash_vec = block.block_hash.clone();
                        let block_hash = bitcoin::block::BlockHash::from_slice(&block_hash_vec)?;
                        let btc_block = self.bitcoin_client_proxy.get_block(block_hash).await?;
                        let block_body = BitcoinBlock::from(btc_block);
                        Some(L1BlockWithBody::new(block.clone(), block_body.encode()))
                    }
                    _ => None,
                };

                tx.send(ExecMsg {
                    tx_order,
                    ledger_tx,
                    l1_block_with_body,
                })
                .await?;
                produced_tx_order = tx_order;
                self.produced
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            next_block_number += 1;
            }
                }
        }
        info!(
            "All transactions are produced, max_block_number: {}, max_tx_order: {}",
            next_block_number, produced_tx_order
        );
        Ok(())
    }

    fn print_tx_cost_stats(hist: &Histogram<u64>, tx_type_str: &str, verbos: bool) {
        let min_size = hist.min() as f64 / 1000f64;
        let max_size = hist.max() as f64 / 1000f64;
        let mean_size = hist.mean() / 1000f64;

        info!(
            "cost stats: {} (ms), count={}, min={}, max={}, mean={:.2}, stdev={:.2}",
            tx_type_str,
            hist.len(),
            min_size,
            max_size,
            mean_size,
            hist.stdev()
        );

        if !verbos {
            return;
        }

        println!(
            "-----------------{} cost percentiles distribution(ms)-----------------",
            tx_type_str
        );
        let percentiles = [
            1.00, 5.00, 10.00, 20.00, 30.00, 40.00, 50.00, 60.00, 70.00, 80.00, 90.00, 95.00,
            99.00, 99.50, 99.90, 99.95, 99.99,
        ];
        let percentile_rows = percentiles.chunks(4);
        for row in percentile_rows {
            let values: Vec<String> = row
                .iter()
                .map(|&p| {
                    let v = hist.value_at_percentile(p) as f64 / 1000f64;
                    format!("{:6.2}th=[{:9.2}]", p, v)
                })
                .collect();
            println!("| {}", values.join(", "));
        }
    }

    async fn consume_tx(&self, mut rx: Receiver<ExecMsg>) -> anyhow::Result<()> {
        info!("Start to consume transactions");
        let mut executed_tx_order = 0;
        let mut interval_cost: u64 = 0;

        const STATISTICS_INTERVAL: u64 = 100000;

        let mut hist_l1block = Histogram::<u64>::new_with_bounds(256, 1 << 30, 3)?;
        let mut hist_l1tx = Histogram::<u64>::new_with_bounds(256, 1 << 30, 3)?;
        let mut hist_l2tx = Histogram::<u64>::new_with_bounds(256, 1 << 30, 3)?;

        // for auto b-search first mismatched tx_order as start point
        let mut last_eq_tx_order = None;

        loop {
            let exec_msg_opt = rx.recv().await;
            if exec_msg_opt.is_none() {
                break;
            }
            let exec_msg = exec_msg_opt.unwrap();
            let tx_order = exec_msg.tx_order;

            let tx_type = match &exec_msg.ledger_tx.data {
                LedgerTxData::L1Block(_) => "L1Block",
                LedgerTxData::L1Tx(_) => "L1Tx",
                LedgerTxData::L2Tx(_) => "L2Tx",
            };

            let elapsed = std::time::Instant::now();
            self.execute(exec_msg, &mut last_eq_tx_order)
                .await
                .with_context(|| {
                    format!(
                        "Error occurs: tx_order: {}, executed_tx_order: {}",
                        tx_order, executed_tx_order
                    )
                })?;
            let tx_cost = elapsed.elapsed().as_micros() as u64;
            interval_cost += tx_cost;

            match tx_type {
                "L1Block" => {
                    hist_l1block.record(tx_cost)?;
                }
                "L1Tx" => {
                    hist_l1tx.record(tx_cost)?;
                }
                "L2Tx" => {
                    hist_l2tx.record(tx_cost)?;
                }
                _ => {}
            }

            executed_tx_order = tx_order;
            self.executed_tx_order
                .store(executed_tx_order, std::sync::atomic::Ordering::Relaxed);
            let done = self.done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

            if done % STATISTICS_INTERVAL == 0 {
                info!(
                    "tx range: [{}, {}], avg: {:.3} ms/tx",
                    tx_order + 1 - STATISTICS_INTERVAL, // add first, avoid overflow
                    tx_order,
                    interval_cost as f64 / 1000.0 / STATISTICS_INTERVAL as f64
                );
                interval_cost = 0;
                Self::print_tx_cost_stats(&hist_l1block, "L1Block", false);
                Self::print_tx_cost_stats(&hist_l1tx, "L1Tx", false);
                Self::print_tx_cost_stats(&hist_l2tx, "L2Tx", false);
            }
        }
        if let Some(verify_targets) = self.mode.get_verify_targets_str(self.bypass_verify) {
            info!(
                "All transactions {} are strictly equal to RoochNetwork: [0, {}]",
                verify_targets, executed_tx_order
            );
        }

        Self::print_tx_cost_stats(&hist_l1block, "L1Block", true);
        Self::print_tx_cost_stats(&hist_l1tx, "L1Tx", true);
        Self::print_tx_cost_stats(&hist_l2tx, "L2Tx", true);
        Ok(())
    }

    async fn execute(
        &self,
        msg: ExecMsg,
        last_eq_tx_order: &mut Option<u64>,
    ) -> anyhow::Result<()> {
        let ExecMsg {
            tx_order,
            mut ledger_tx,
            l1_block_with_body,
        } = msg;
        let tx_hash = ledger_tx.tx_hash();
        let is_l2_tx = ledger_tx.data.is_l2_tx();

        let mut bypass_execution = false;
        if let Some(tx_anomalies) = &self.tx_anomalies {
            if tx_anomalies.has_no_execution_info(&tx_hash) {
                bypass_execution = true;
            }
        }

        // it's okay to sequence tx before validation,
        // because in this case, all tx have been sequenced in Rooch Network.
        if self.mode.need_seq() {
            self.sequenced_tx_store.save_tx(ledger_tx.clone())?;
        }

        if self.mode.need_exec() && !bypass_execution {
            let moveos_tx = self
                .validate_ledger_transaction(ledger_tx, l1_block_with_body)
                .await?;
            if let Err(err) = self
                .execute_moveos_tx(tx_order, moveos_tx, last_eq_tx_order)
                .await
            {
                self.handle_execution_error(err, is_l2_tx, tx_order)?;
            }
        }

        Ok(())
    }

    fn handle_execution_error(
        &self,
        error: anyhow::Error,
        is_l2_tx: bool,
        tx_order: u64,
    ) -> anyhow::Result<()> {
        if is_l2_tx {
            return if is_vm_panic_error(&error) {
                tracing::error!(
                    "Execute L2 Tx failed while VM panic occurred. tx_order: {}, error: {:?}",
                    tx_order,
                    error,
                );
                Err(error)
            } else {
                // return error if is state root not equal to RoochNetwork
                if error
                    .to_string()
                    .contains("Execution state root is not equal to RoochNetwork")
                {
                    return Err(error);
                }

                warn!(
                    "Execute L2 Tx failed while non-VM panic occurred. tx_order: {}, error: {:?}",
                    tx_order, error
                );
                Err(error) // return every error
            };
        }

        // Default error handling for non-L2Tx transactions and other cases.
        Err(error)
    }

    async fn validate_ledger_transaction(
        &self,
        ledger_tx: LedgerTransaction,
        l1block_with_body: Option<L1BlockWithBody>,
    ) -> anyhow::Result<VerifiedMoveOSTransaction> {
        let tx_order = ledger_tx.sequence_info.tx_order;
        let bypass_l1_executed_check = tx_order < self.check_l1_tx_executed_start_from;

        let moveos_tx_result = match &ledger_tx.data {
            LedgerTxData::L1Block(_block) => {
                self.executor
                    .validate_l1_block(l1block_with_body.unwrap())
                    .await
            }
            LedgerTxData::L1Tx(l1_tx) => {
                self.executor
                    .validate_l1_tx(l1_tx.clone(), bypass_l1_executed_check)
                    .await
            }
            LedgerTxData::L2Tx(l2_tx) => self.executor.validate_l2_tx(l2_tx.clone()).await,
        };

        let mut moveos_tx = match moveos_tx_result {
            Ok(tx) => tx,
            Err(err) => {
                tracing::error!(
                    "Error validating transaction: tx_order: {}, error: {:?}",
                    ledger_tx.sequence_info.tx_order,
                    err
                );
                return Err(err);
            }
        };

        moveos_tx.ctx.add(ledger_tx.sequence_info)?;
        Ok(moveos_tx)
    }

    async fn execute_moveos_tx(
        &self,
        tx_order: u64,
        moveos_tx: VerifiedMoveOSTransaction,
        last_eq_tx_order: &mut Option<u64>,
    ) -> anyhow::Result<()> {
        let executor = self.executor.clone();

        let (_output, execution_info) = executor.execute_transaction(moveos_tx.clone()).await?;

        let exp_state_root = if self.bypass_verify {
            None
        } else {
            self.tx_meta_store.get_exp_state_root(tx_order).await
        };

        let root = execution_info.root_metadata();
        match exp_state_root {
            Some(expected_root) => {
                if root.state_root.unwrap() != expected_root {
                    if let Some(last_eq_value) = last_eq_tx_order {
                        let mid_tx_order = (*last_eq_value + tx_order) / 2;
                        self.state_root_fetcher.fetch_and_add(mid_tx_order).await?;
                        info!("state root of tx_order: {} fetched, it's in the middle of last_eq_tx_order: {} and first not_eq_tx_order: {}", mid_tx_order, *last_eq_value, tx_order);
                    }

                    return Err(anyhow::anyhow!(
                        "Execution state root is not equal to RoochNetwork: tx_order: {}, exp: {:?}, act: {:?}; act_execution_info: {:?}. Please rollback to last_eq_tx_order: {:?}",
                        tx_order,
                        expected_root, root.state_root.unwrap(), execution_info, last_eq_tx_order
                    ));
                }
                info!(
                    "Execution state root is equal to RoochNetwork: tx_order: {}; last_eq_tx_order: {:?}",
                    tx_order, last_eq_tx_order
                );
                *last_eq_tx_order = Some(tx_order);
                Ok(())
            }
            None => Ok(()),
        }
    }
}

async fn build_btc_client_proxy(
    btc_rpc_url: String,
    btc_rpc_user_name: String,
    btc_rpc_password: String,
    btc_local_block_store_dir: Option<PathBuf>,
    actor_system: &ActorSystem,
) -> anyhow::Result<BitcoinClientProxy> {
    let bitcoin_client_config = BitcoinClientConfig {
        btc_rpc_url,
        btc_rpc_user_name,
        btc_rpc_password,
        local_block_store_dir: btc_local_block_store_dir,
    };

    let bitcoin_client = bitcoin_client_config.build()?;
    let bitcoin_client_actor_ref = bitcoin_client
        .into_actor(Some("bitcoin_client_for_rpc_service"), actor_system)
        .await?;
    Ok(BitcoinClientProxy::new(bitcoin_client_actor_ref.into()))
}

async fn build_executor_and_store(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
    actor_system: &ActorSystem,
    enable_rocks_stats: bool,
    row_cache_size: Option<u64>,
    block_cache_size: Option<u64>,
) -> anyhow::Result<(ExecutorProxy, MoveOSStore, RoochDB)> {
    let registry_service = RegistryService::default();

    let (root, rooch_db) = build_rooch_db(
        base_data_dir.clone(),
        chain_id.clone(),
        enable_rocks_stats,
        row_cache_size,
        block_cache_size,
    );
    let (rooch_store, moveos_store) = (rooch_db.rooch_store.clone(), rooch_db.moveos_store.clone());

    let event_bus = EventBus::new();
    let subscription_handle = Arc::new(SubscriptionHandler::new(
        &registry_service.default_registry(),
    ));
    let notify_actor = NotifyActor::new(event_bus.clone(), subscription_handle);
    let notify_actor_ref = notify_actor
        .into_actor(Some("NotifyActor"), actor_system)
        .await?;

    let executor_actor = ExecutorActor::new(
        root.clone(),
        moveos_store.clone(),
        rooch_store.clone(),
        &registry_service.default_registry(),
        Some(notify_actor_ref.clone()),
    )?;

    let executor_actor_ref = executor_actor
        .into_actor(Some("Executor"), actor_system)
        .await?;

    let reader_executor = ReaderExecutorActor::new(
        root.clone(),
        moveos_store.clone(),
        rooch_store.clone(),
        None,
    )?;

    let read_executor_ref = reader_executor
        .into_actor(Some("ReadExecutor"), actor_system)
        .await?;

    Ok((
        ExecutorProxy::new(
            executor_actor_ref.clone().into(),
            read_executor_ref.clone().into(),
        ),
        moveos_store,
        rooch_db,
    ))
}

#[cfg(test)]
mod tests {
    use crate::commands::da::commands::exec::ExecMode;

    #[test]
    fn test_exec_mode() {
        let mode = ExecMode::Exec;
        assert!(mode.need_exec());
        assert!(!mode.need_seq());
        assert!(!mode.need_all());

        let mode = ExecMode::Seq;
        assert!(!mode.need_exec());
        assert!(mode.need_seq());
        assert!(!mode.need_all());

        let mode = ExecMode::All;
        assert!(mode.need_exec());
        assert!(mode.need_seq());
        assert!(mode.need_all());

        let mode = ExecMode::Sync;
        assert!(mode.need_exec());
        assert!(mode.need_seq());
        assert!(mode.need_all());
        assert!(mode.need_sync());

        let mode = ExecMode::SyncExec;
        assert!(mode.need_exec());
        assert!(!mode.need_seq());
        assert!(!mode.need_all());
        assert!(mode.need_sync())
    }
}
