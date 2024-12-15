// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::{collect_chunks, get_tx_list_from_chunk};
use bitcoin::hashes::Hash;
use bitcoin_client::actor::client::BitcoinClientConfig;
use bitcoin_client::proxy::BitcoinClientProxy;
use clap::Parser;
use coerce::actor::system::ActorSystem;
use coerce::actor::IntoActor;
use metrics::RegistryService;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use rooch_config::RoochOpt;
use rooch_config::R_OPT_NET_HELP;
use rooch_db::RoochDB;
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
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

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
}

async fn build_btc_client_proxy(
    btc_rpc_url: String,
    btc_rpc_user_name: String,
    btc_rpc_password: String,
    actor_system: &ActorSystem,
) -> anyhow::Result<BitcoinClientProxy> {
    let bitcoin_client_config = BitcoinClientConfig {
        btc_rpc_url,
        btc_rpc_user_name,
        btc_rpc_password,
    };

    let bitcoin_client = bitcoin_client_config.build()?;
    let bitcoin_client_actor_ref = bitcoin_client
        .into_actor(Some("bitcoin_client_for_rpc_service"), actor_system)
        .await?;
    Ok(BitcoinClientProxy::new(bitcoin_client_actor_ref.into()))
}

fn build_rooch_db(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> (ObjectMeta, RoochDB) {
    let opt = RoochOpt::new_with_default(base_data_dir, chain_id, None).unwrap();
    let registry_service = RegistryService::default();
    let rooch_db = RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();
    let root = rooch_db.latest_root().unwrap().unwrap();
    (root, rooch_db)
}

async fn build_executor_proxy(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
    actor_system: &ActorSystem,
) -> anyhow::Result<ExecutorProxy> {
    let registry_service = RegistryService::default();

    let (root, rooch_db) = build_rooch_db(base_data_dir.clone(), chain_id.clone());
    let (rooch_store, moveos_store) = (rooch_db.rooch_store.clone(), rooch_db.moveos_store.clone());

    let executor_actor = ExecutorActor::new(
        root.clone(),
        moveos_store.clone(),
        rooch_store.clone(),
        &registry_service.default_registry(),
        None,
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

    Ok(ExecutorProxy::new(
        executor_actor_ref.clone().into(),
        read_executor_ref.clone().into(),
    ))
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
            &actor_system,
        )
        .await?;
        let executor = build_executor_proxy(
            self.base_data_dir.clone(),
            self.chain_id.clone(),
            &actor_system,
        )
        .await?;

        let (order_state_pair, tx_order_end) = self.load_order_state_pair();
        let chunks = collect_chunks(self.segment_dir.clone())?;
        Ok(ExecInner {
            segment_dir: self.segment_dir.clone(),
            chunks,
            order_state_pair,
            tx_order_end,
            bitcoin_client_proxy,
            executor,
        })
    }

    fn load_order_state_pair(&self) -> (HashMap<u64, H256>, u64) {
        let mut order_state_pair = HashMap::new();
        let mut tx_order_end = 0;

        let mut reader = BufReader::new(File::open(self.order_state_path.clone()).unwrap());
        // collect all tx_order:state_root pairs
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
    segment_dir: PathBuf,
    chunks: HashMap<u128, Vec<u64>>,
    order_state_pair: HashMap<u64, H256>,
    tx_order_end: u64,

    bitcoin_client_proxy: BitcoinClientProxy,
    pub(crate) executor: ExecutorProxy,
}

impl ExecInner {
    async fn run(&self) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let producer = self.produce_tx(tx);
        let consumer = self.consume_tx(rx);

        let (producer_result, consumer_result) = tokio::join!(producer, consumer);
        producer_result?;
        consumer_result?;

        Ok(())
    }

    async fn produce_tx(&self, tx: Sender<LedgerTransaction>) -> anyhow::Result<()> {
        let mut block_number = 0;
        let mut produced_tx_order = 0;
        let mut executed = true;
        loop {
            let tx_list = self.load_ledger_tx_list(block_number)?;
            if tx_list.is_none() {
                block_number -= 1; // no chunk belongs to this block_number
                break;
            }
            let tx_list = tx_list.unwrap();
            for mut ledger_tx in tx_list {
                let tx_order = ledger_tx.sequence_info.tx_order;
                if tx_order > self.tx_order_end {
                    break;
                }
                if executed {
                    let execution_info = self
                        .executor
                        .get_transaction_execution_infos_by_hash(vec![ledger_tx.data.tx_hash()])
                        .await?;
                    if !execution_info.is_empty() {
                        continue;
                    }
                    executed = false;
                }

                tx.send(ledger_tx).await?;
                produced_tx_order = tx_order;
            }
            block_number += 1;
        }
        drop(tx);
        println!(
            "All transactions are produced, max_block_number: {}, max_tx_order: {}",
            block_number, produced_tx_order
        );
        Ok(())
    }

    async fn consume_tx(&self, mut rx: Receiver<LedgerTransaction>) -> anyhow::Result<()> {
        let mut verified_tx_order = 0;
        loop {
            let ledger_tx = rx.recv().await;
            if ledger_tx.is_none() {
                break;
            }
            let ledger_tx = ledger_tx.unwrap();
            let tx_order = ledger_tx.sequence_info.tx_order;
            self.execute_ledger_tx(tx_order, ledger_tx).await?;
            verified_tx_order = tx_order;
        }
        println!(
            "All transactions execution state root are strictly equal to RoochNetwork: [0, {}]",
            verified_tx_order
        );
        Ok(())
    }

    fn load_ledger_tx_list(
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

    async fn execute_ledger_tx(
        &self,
        tx_order: u64,
        ledger_tx: LedgerTransaction,
    ) -> anyhow::Result<()> {
        let executor = self.executor.clone();
        let bitcoin_client_proxy = self.bitcoin_client_proxy.clone();

        let mut moveos_tx = match &ledger_tx.data {
            LedgerTxData::L1Block(block) => {
                let block_hash_vec = block.block_hash.clone();
                let block_hash = bitcoin::block::BlockHash::from_slice(&block_hash_vec)?;
                let btc_block = bitcoin_client_proxy.get_block(block_hash).await?;
                let block_body = BitcoinBlock::from(btc_block);
                executor
                    .validate_l1_block(L1BlockWithBody::new(block.clone(), block_body.encode()))
                    .await?
            }
            LedgerTxData::L1Tx(l1_tx) => executor.validate_l1_tx(l1_tx.clone()).await?,
            LedgerTxData::L2Tx(l2_tx) => executor.validate_l2_tx(l2_tx.clone()).await?,
        };
        moveos_tx.ctx.add(ledger_tx.sequence_info.clone())?;
        self.execute_moveos_tx(tx_order, moveos_tx).await?;

        Ok(())
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
                        "Execution state root is not equal to RoochNetwork: tx_order: {}, exp: {:?}, act: {:?}",
                        tx_order,
                        *expected_root, root
                    ));
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
}
