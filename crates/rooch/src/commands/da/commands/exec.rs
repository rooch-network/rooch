// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::{build_rooch_db, LedgerTxGetter, TxDAIndexer};
use anyhow::Context;
use bitcoin::hashes::Hash;
use bitcoin_client::actor::client::BitcoinClientConfig;
use bitcoin_client::proxy::BitcoinClientProxy;
use clap::Parser;
use coerce::actor::system::ActorSystem;
use coerce::actor::IntoActor;
use metrics::RegistryService;
use moveos_common::utils::to_bytes;
use moveos_eventbus::bus::EventBus;
use moveos_store::config_store::STARTUP_INFO_KEY;
use moveos_store::{MoveOSStore, CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME};
use moveos_types::h256::H256;
use moveos_types::startup_info;
use moveos_types::transaction::{TransactionExecutionInfo, VerifiedMoveOSTransaction};
use raw_store::rocks::batch::WriteBatch;
use raw_store::traits::DBStore;
use rooch_config::R_OPT_NET_HELP;
use rooch_db::RoochDB;
use rooch_event::actor::EventActor;
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_types::bitcoin::types::Block as BitcoinBlock;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use rooch_types::transaction::{L1BlockWithBody, LedgerTransaction, LedgerTxData};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio::time;

/// exec LedgerTransaction List for verification.
#[derive(Debug, Parser)]
pub struct ExecCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(
        long = "order-state-path",
        help = "Path to tx_order:state_root file(results from RoochNetwork), for verification"
    )]
    pub order_state_path: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long = "btc-rpc-url")]
    pub btc_rpc_url: String,
    #[clap(long = "btc-rpc-user-name")]
    pub btc_rpc_user_name: String,
    #[clap(long = "btc-rpc-password")]
    pub btc_rpc_password: String,
    #[clap(long = "btc-local-block-store-dir")]
    pub btc_local_block_store_dir: Option<PathBuf>,

    #[clap(name = "rocksdb-row-cache-size", long, help = "rocksdb row cache size")]
    pub row_cache_size: Option<u64>,

    #[clap(
        name = "rocksdb-block-cache-size",
        long,
        help = "rocksdb block cache size"
    )]
    pub block_cache_size: Option<u64>,
    #[clap(long = "enable-rocks-stats", help = "rocksdb-enable-statistics")]
    pub enable_rocks_stats: bool,

    #[clap(
        long = "order-hash-path",
        help = "Path to tx_order:tx_hash:block_number file"
    )]
    pub order_hash_path: PathBuf,
    #[clap(
        long = "rollback",
        help = "rollback to tx order. If not set or ge executed_tx_order, start from executed_tx_order+1(nothing to do); otherwise, rollback to this order."
    )]
    pub rollback: Option<u64>,
}

impl ExecCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let exec_inner = self.build_exec_inner().await?;
        exec_inner.run().await?;
        Ok(())
    }

    async fn build_exec_inner(&self) -> anyhow::Result<ExecInner> {
        let actor_system = ActorSystem::global_system();
        let bitcoin_client_proxy = build_btc_client_proxy(
            self.btc_rpc_url.clone(),
            self.btc_rpc_user_name.clone(),
            self.btc_rpc_password.clone(),
            self.btc_local_block_store_dir.clone(),
            &actor_system,
        )
        .await?;
        let (executor, moveos_store, rooch_db) = build_executor_and_store(
            self.base_data_dir.clone(),
            self.chain_id.clone(),
            &actor_system,
            self.enable_rocks_stats,
            self.row_cache_size,
            self.block_cache_size,
        )
        .await?;

        let (order_state_pair, tx_order_end) = self.load_order_state_pair();
        let ledger_tx_loader = LedgerTxGetter::new(self.segment_dir.clone())?;
        let tx_da_indexer = TxDAIndexer::load_from_file(
            self.order_hash_path.clone(),
            moveos_store.transaction_store,
        )?;
        Ok(ExecInner {
            ledger_tx_getter: ledger_tx_loader,
            tx_da_indexer,
            order_state_pair,
            tx_order_end,
            bitcoin_client_proxy,
            executor,
            produced: Arc::new(AtomicU64::new(0)),
            done: Arc::new(AtomicU64::new(0)),
            executed_tx_order: Arc::new(AtomicU64::new(0)),
            rollback: self.rollback,
            rooch_db,
        })
    }

    fn load_order_state_pair(&self) -> (HashMap<u64, H256>, u64) {
        let mut order_state_pair = HashMap::new();
        let mut tx_order_end = 0;

        let mut reader = BufReader::new(File::open(self.order_state_path.clone()).unwrap());
        // collect all `tx_order:state_root` pairs
        for line in reader.by_ref().lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split(':').collect();
            let tx_order = parts[0].parse::<u64>().unwrap();
            let state_root = H256::from_str(parts[1]).unwrap();
            order_state_pair.insert(tx_order, state_root);
            if tx_order > tx_order_end {
                tx_order_end = tx_order;
            }
        }
        (order_state_pair, tx_order_end)
    }
}

struct ExecInner {
    ledger_tx_getter: LedgerTxGetter,
    tx_da_indexer: TxDAIndexer,
    order_state_pair: HashMap<u64, H256>,
    tx_order_end: u64,

    bitcoin_client_proxy: BitcoinClientProxy,
    executor: ExecutorProxy,

    rooch_db: RoochDB,
    rollback: Option<u64>,

    // stats
    produced: Arc<AtomicU64>,
    done: Arc<AtomicU64>,
    executed_tx_order: Arc<AtomicU64>,
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
                        tracing::info!("Shutting down logging task.");
                        break;
                    }
                    _ = interval.tick() => {
                        let done = done_cloned.load(std::sync::atomic::Ordering::Relaxed);
                        let executed_tx_order = executed_tx_order_cloned.load(std::sync::atomic::Ordering::Relaxed);
                        let produced = produced_cloned.load(std::sync::atomic::Ordering::Relaxed);

                        tracing::info!(
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

    async fn run(&self) -> anyhow::Result<()> {
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        self.start_logging_task(shutdown_rx);

        // larger buffer size to avoid rx starving caused by consumer has to access disks and request btc block.
        // after consumer load data(ledger_tx) from disk/btc client, burst to executor, need large buffer to avoid blocking.
        // 16384 is a magic number, it's a trade-off between memory usage and performance. (usually tx count inside a block is under 8192, MAX_TXS_PER_BLOCK_IN_FIX)
        let (tx, rx) = tokio::sync::mpsc::channel(16384);
        let producer = self.produce_tx(tx);
        let consumer = self.consume_tx(rx);

        let result = self.join_producer_and_consumer(producer, consumer).await;

        // Send shutdown signal and ensure logging task exits
        let _ = shutdown_tx.send(());
        result
    }

    fn update_startup_info_after_rollback(
        &self,
        execution_info: TransactionExecutionInfo,
    ) -> anyhow::Result<()> {
        let rollback_startup_info =
            startup_info::StartupInfo::new(execution_info.state_root, execution_info.size);

        let inner_store = &self.rooch_db.rooch_store.store_instance;
        let mut write_batch = WriteBatch::new();
        let cf_names = vec![CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME];

        write_batch.put(
            to_bytes(STARTUP_INFO_KEY).unwrap(),
            to_bytes(&rollback_startup_info).unwrap(),
        )?;

        inner_store.write_batch_across_cfs(cf_names, write_batch, true)
    }

    async fn produce_tx(&self, tx: Sender<ExecMsg>) -> anyhow::Result<()> {
        let last_executed_opt = self.tx_da_indexer.find_last_executed()?;
        let next_tx_order = last_executed_opt
            .clone()
            .map(|v| v.tx_order + 1)
            .unwrap_or(1);
        let mut next_block_number = last_executed_opt
            .clone()
            .map(|v| v.block_number) // next_tx_order and last executed tx may be in the same block
            .unwrap_or(0);
        tracing::info!(
            "next_tx_order: {:?}. need rollback soon: {:?}",
            next_tx_order,
            self.rollback.is_some()
        );

        // If rollback not set or ge executed_tx_order, start from executed_tx_order+1(nothing to do); otherwise, rollback to this order
        if let (Some(rollback), Some(last_executed)) = (self.rollback, last_executed_opt.clone()) {
            let last_executed_tx_order = last_executed.tx_order;
            if rollback < last_executed_tx_order {
                let new_last_and_rollback =
                    self.tx_da_indexer.slice(rollback, last_executed_tx_order)?;
                // split into two parts, the first get execution info for new startup, all others rollback
                let (new_last, rollback_part) = new_last_and_rollback.split_first().unwrap();
                tracing::info!(
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
                    self.tx_da_indexer.get_execution_info(new_last.tx_hash)?;
                self.update_startup_info_after_rollback(rollback_execution_info.unwrap())?;
                tracing::info!(
                    "Rollback transactions done. Please RESTART process without rollback."
                );
                return Ok(()); // rollback done, need to restart to get new state_root for startup rooch store
            }
        };

        tracing::info!(
            "Start to produce transactions from tx_order: {}, check from block: {}",
            next_tx_order,
            next_block_number,
        );
        let mut produced_tx_order = 0;
        let mut reach_end = false;
        loop {
            if reach_end {
                break;
            }
            let tx_list = self
                .ledger_tx_getter
                .load_ledger_tx_list(next_block_number, false)?;
            if tx_list.is_none() {
                next_block_number -= 1; // no chunk belongs to this block_number
                break;
            }
            let tx_list = tx_list.unwrap();
            for ledger_tx in tx_list {
                let tx_order = ledger_tx.sequence_info.tx_order;
                if tx_order > self.tx_order_end {
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
        tracing::info!(
            "All transactions are produced, max_block_number: {}, max_tx_order: {}",
            next_block_number,
            produced_tx_order
        );
        Ok(())
    }

    async fn consume_tx(&self, mut rx: Receiver<ExecMsg>) -> anyhow::Result<()> {
        tracing::info!("Start to consume transactions");
        let mut executed_tx_order = 0;
        let mut last_record_time = std::time::Instant::now();
        loop {
            let exec_msg_opt = rx.recv().await;
            if exec_msg_opt.is_none() {
                break;
            }
            let exec_msg = exec_msg_opt.unwrap();
            let tx_order = exec_msg.tx_order;

            self.execute(exec_msg).await.with_context(|| {
                format!(
                    "Error executing transaction: tx_order: {}, executed_tx_order: {}",
                    tx_order, executed_tx_order
                )
            })?;

            executed_tx_order = tx_order;
            self.executed_tx_order
                .store(executed_tx_order, std::sync::atomic::Ordering::Relaxed);
            let done = self.done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

            if done % 10000 == 0 {
                let elapsed = last_record_time.elapsed();
                tracing::info!(
                    "execute tx range: [{}, {}], cost: {:?}, avg: {:.3} ms/tx",
                    tx_order + 1 - 10000, // add first, avoid overflow
                    tx_order,
                    elapsed,
                    elapsed.as_millis() as f64 / 10000f64
                );
                last_record_time = std::time::Instant::now();
            }
        }
        tracing::info!(
            "All transactions execution state root are strictly equal to RoochNetwork: [0, {}]",
            executed_tx_order
        );
        Ok(())
    }

    async fn execute(&self, msg: ExecMsg) -> anyhow::Result<()> {
        let ExecMsg {
            tx_order,
            ledger_tx,
            l1_block_with_body,
        } = msg;
        let moveos_tx = self
            .validate_ledger_transaction(ledger_tx, l1_block_with_body)
            .await?;
        self.execute_moveos_tx(tx_order, moveos_tx).await
    }

    async fn validate_ledger_transaction(
        &self,
        ledger_tx: LedgerTransaction,
        l1block_with_body: Option<L1BlockWithBody>,
    ) -> anyhow::Result<VerifiedMoveOSTransaction> {
        let mut moveos_tx = match &ledger_tx.data {
            LedgerTxData::L1Block(_block) => {
                self.executor
                    .validate_l1_block(l1block_with_body.unwrap())
                    .await?
            }
            LedgerTxData::L1Tx(l1_tx) => self.executor.validate_l1_tx(l1_tx.clone()).await?,
            LedgerTxData::L2Tx(l2_tx) => self.executor.validate_l2_tx(l2_tx.clone()).await?,
        };
        moveos_tx.ctx.add(ledger_tx.sequence_info.clone())?;
        Ok(moveos_tx)
    }

    async fn execute_moveos_tx(
        &self,
        tx_order: u64,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> anyhow::Result<()> {
        let executor = self.executor.clone();

        let (_output, execution_info) = executor.execute_transaction(moveos_tx.clone()).await?;

        let root = execution_info.root_metadata();
        let expected_root_opt = self.order_state_pair.get(&tx_order);
        match expected_root_opt {
            Some(expected_root) => {
                if root.state_root.unwrap() != *expected_root {
                    return Err(anyhow::anyhow!(
                        "Execution state root is not equal to RoochNetwork: tx_order: {}, exp: {:?}, act: {:?}; act_execution_info: {:?}",
                        tx_order,
                        *expected_root, root.state_root.unwrap(), execution_info
                    ));
                }
                tracing::info!(
                    "Execution state root is equal to RoochNetwork: tx_order: {}",
                    tx_order
                );
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
    let event_actor = EventActor::new(event_bus.clone());
    let event_actor_ref = event_actor
        .into_actor(Some("EventActor"), actor_system)
        .await?;

    let executor_actor = ExecutorActor::new(
        root.clone(),
        moveos_store.clone(),
        rooch_store.clone(),
        &registry_service.default_registry(),
        Some(event_actor_ref.clone()),
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
