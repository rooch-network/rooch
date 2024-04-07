// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::tx::TxType::{BTCBlk, Blog, Empty, Transfer};
use anyhow::Result;
use bitcoin::consensus::deserialize;
use bitcoin::hashes::Hash;
use bitcoin::hex::FromHex;
use bitcoincore_rpc_json::bitcoin;
use bitcoincore_rpc_json::bitcoin::Block;
use coerce::actor::scheduler::timer::Timer;
use coerce::actor::system::ActorSystem;
use coerce::actor::IntoActor;
use criterion::Criterion;
use lazy_static::lazy_static;
use moveos_config::store_config::RocksdbConfig;
use moveos_config::DataDirPath;
use moveos_store::{MoveOSDB, MoveOSStore};
use raw_store::rocks::RocksDB;
use raw_store::StoreInstance;
use rooch_config::da_config::DAConfig;
use rooch_config::indexer_config::IndexerConfig;
use rooch_config::store_config::StoreConfig;
use rooch_da::actor::da::DAActor;
use rooch_da::proxy::DAProxy;
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_framework::natives::default_gas_schedule;
use rooch_framework_tests::binding_test;
use rooch_indexer::actor::indexer::IndexerActor;
use rooch_indexer::actor::reader_indexer::IndexerReaderActor;
use rooch_indexer::indexer_reader::IndexerReader;
use rooch_indexer::proxy::IndexerProxy;
use rooch_indexer::IndexerStore;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::memory_keystore::InMemKeystore;
use rooch_pipeline_processor::{
    actor::processor::PipelineProcessorActor, proxy::PipelineProcessorProxy,
};
use rooch_proposer::actor::messages::ProposeBlock;
use rooch_proposer::actor::proposer::ProposerActor;
use rooch_proposer::proxy::ProposerProxy;
use rooch_rpc_server::service::aggregate_service::AggregateService;
use rooch_rpc_server::service::rpc_service::RpcService;
use rooch_sequencer::actor::sequencer::SequencerActor;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_store::RoochStore;
use rooch_test_transaction_builder::TestTransactionBuilder;
use rooch_types::address::RoochAddress;
use rooch_types::bitcoin::data_import_config::DataImportMode;
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::bitcoin::network::Network;
use rooch_types::chain_id::RoochChainID;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::transaction::L1BlockWithBody;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tempfile::TempDir;
use tracing::info;

pub const EXAMPLE_SIMPLE_BLOG_PACKAGE_NAME: &'static str = "simple_blog";
pub const EXAMPLE_SIMPLE_BLOG_NAMED_ADDRESS: &str = "simple_blog";

#[derive(PartialEq, Eq)]
pub enum TxType {
    Empty,
    Transfer,
    Blog,
    BTCBlk,
}

impl FromStr for TxType {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "transfer" => Ok(TxType::Transfer),
            "blog" => Ok(TxType::Blog),
            "btc_block" => Ok(TxType::BTCBlk),
            _ => Ok(TxType::Empty),
        }
    }
}

impl Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TxType::Empty => "empty".to_string(),
            TxType::Transfer => "transfer".to_string(),
            TxType::Blog => "blog".to_string(),
            TxType::BTCBlk => "btc_blk".to_string(),
        };
        write!(f, "{}", str)
    }
}

lazy_static! {
    pub static ref TX_SIZE: usize = {
        env::var("ROOCH_BENCH_TX_SIZE")
            .unwrap_or_else(|_| String::from("0"))
            .parse::<usize>()
            .unwrap_or(0usize)
    };
    pub static ref TX_TYPE: TxType = {
        let tx_type_str = env::var("ROOCH_BENCH_TX_TYPE").unwrap_or_else(|_| String::from("empty"));
        tx_type_str.parse::<TxType>().unwrap_or(TxType::Empty)
    };
    pub static ref DATA_DIR: DataDirPath = get_data_dir();
    pub static ref BTC_BLK_DIR: String =
        env::var("ROOCH_BENCH_BTC_BLK_DIR").unwrap_or(String::from("data/btc"));
}

pub fn get_data_dir() -> DataDirPath {
    match env::var("ROOCH_TEST_DATA_DIR") {
        Ok(path_str) => {
            let temp_dir = TempDir::new_in(path_str)
                .expect("Failed to create temp dir in provided data dir path");
            DataDirPath::TempPath(Arc::from(temp_dir))
        }
        Err(_) => moveos_config::temp_dir(),
    }
}

pub fn gen_sequencer(keypair: RoochKeyPair, rooch_store: RoochStore) -> Result<SequencerActor> {
    SequencerActor::new(keypair, rooch_store.clone(), true) // is_genesis is useless for sequencer in present
}

//TODO reuse the rpc run_start_server function
pub async fn setup_service(
    datadir: &DataDirPath,
    keystore: &InMemKeystore,
) -> Result<(RpcService, AggregateService)> {
    // We may call `start_server` multiple times in testing scenarios
    // tracing_subscriber can only be inited once.
    let _ = tracing_subscriber::fmt::try_init();

    let actor_system = ActorSystem::global_system();
    let chain_id = RoochChainID::LOCAL;

    // init storage
    let (moveos_store, rooch_store) = init_storage(datadir)?;
    let (indexer_store, indexer_reader) = init_indexer(datadir)?;

    // init keystore
    let rooch_account = keystore.addresses()[0];
    let rooch_key_pair = keystore
        .get_key_pairs(&rooch_account, None)?
        .pop()
        .expect("Key pair should have value");

    let sequencer_keypair = rooch_key_pair.copy();
    let proposer_keypair = rooch_key_pair.copy();
    let relayer_keypair = rooch_key_pair.copy();
    let sequencer_account = RoochAddress::from(&sequencer_keypair.public());
    let proposer_account = RoochAddress::from(&proposer_keypair.public());
    let _relayer_account = RoochAddress::from(&relayer_keypair.public());

    // Init executor
    let is_genesis = moveos_store.statedb.is_genesis();
    let btc_network = Network::default().to_num();
    let data_import_mode = DataImportMode::default().to_num();
    let gas_schedule_blob =
        bcs::to_bytes(&default_gas_schedule()).expect("Failure serializing genesis gas schedule");
    let executor_actor = ExecutorActor::new(
        chain_id.genesis_ctx(rooch_account, gas_schedule_blob),
        BitcoinGenesisContext::new(btc_network, data_import_mode),
        moveos_store.clone(),
        rooch_store.clone(),
    )?;
    let reader_executor = ReaderExecutorActor::new(
        executor_actor.genesis().clone(),
        moveos_store.clone(),
        rooch_store.clone(),
    )?
    .into_actor(Some("ReaderExecutor"), &actor_system)
    .await?;
    let executor = executor_actor
        .into_actor(Some("Executor"), &actor_system)
        .await?;
    let executor_proxy = ExecutorProxy::new(executor.into(), reader_executor.into());

    // Init sequencer
    info!("RPC Server sequencer address: {:?}", sequencer_account);
    let sequencer = SequencerActor::new(sequencer_keypair, rooch_store.clone(), is_genesis)?
        .into_actor(Some("Sequencer"), &actor_system)
        .await?;
    let sequencer_proxy = SequencerProxy::new(sequencer.into());

    // Init DA
    let da_config = DAConfig::default();
    let da_proxy = DAProxy::new(
        DAActor::new(da_config, &actor_system)
            .await?
            .into_actor(Some("DAProxy"), &actor_system)
            .await?
            .into(),
    );

    // Init proposer
    info!("RPC Server proposer address: {:?}", proposer_account);
    let proposer = ProposerActor::new(proposer_keypair, da_proxy)
        .into_actor(Some("Proposer"), &actor_system)
        .await?;
    let proposer_proxy = ProposerProxy::new(proposer.clone().into());
    //TODO load from config
    let block_propose_duration_in_seconds: u64 = 5;
    let mut timers = vec![];
    let proposer_timer = Timer::start(
        proposer,
        Duration::from_secs(block_propose_duration_in_seconds),
        ProposeBlock {},
    );
    timers.push(proposer_timer);

    // Init indexer
    let indexer_executor = IndexerActor::new(indexer_store.clone(), moveos_store.clone())?
        .into_actor(Some("Indexer"), &actor_system)
        .await?;
    let indexer_reader_executor = IndexerReaderActor::new(indexer_reader)?
        .into_actor(Some("IndexerReader"), &actor_system)
        .await?;
    let indexer_proxy = IndexerProxy::new(indexer_executor.into(), indexer_reader_executor.into());

    let processor = PipelineProcessorActor::new(
        executor_proxy.clone(),
        sequencer_proxy.clone(),
        proposer_proxy.clone(),
        indexer_proxy.clone(),
        true,
    )
    .into_actor(Some("PipelineProcessor"), &actor_system)
    .await?;
    let processor_proxy = PipelineProcessorProxy::new(processor.into());

    let rpc_service = RpcService::new(
        executor_proxy.clone(),
        sequencer_proxy,
        indexer_proxy,
        processor_proxy,
    );
    let aggregate_service = AggregateService::new(rpc_service.clone());

    Ok((rpc_service, aggregate_service))
}

pub fn init_storage(datadir: &DataDirPath) -> Result<(MoveOSStore, RoochStore)> {
    let (rooch_db_path, moveos_db_path) = (
        StoreConfig::get_mock_rooch_store_dir(datadir),
        StoreConfig::get_mock_moveos_store_dir(datadir),
    );
    if !rooch_db_path.exists() {
        std::fs::create_dir_all(rooch_db_path.clone())?;
    }
    if !moveos_db_path.exists() {
        std::fs::create_dir_all(moveos_db_path.clone())?;
    }

    //Init store
    let moveosdb = MoveOSDB::new(StoreInstance::new_db_instance(RocksDB::new(
        moveos_db_path,
        moveos_store::StoreMeta::get_column_family_names().to_vec(),
        RocksdbConfig::default(),
        None,
    )?))?;
    let moveos_store = MoveOSStore::new(moveosdb)?;

    let rooch_store = RoochStore::new(StoreInstance::new_db_instance(RocksDB::new(
        rooch_db_path,
        rooch_store::StoreMeta::get_column_family_names().to_vec(),
        RocksdbConfig::default(),
        None,
    )?))?;

    Ok((moveos_store, rooch_store))
}

pub fn init_indexer(datadir: &DataDirPath) -> Result<(IndexerStore, IndexerReader)> {
    let indexer_db_path = IndexerConfig::get_mock_indexer_db(datadir);
    if !indexer_db_path.exists() {
        std::fs::create_dir_all(indexer_db_path.clone())?;
    }
    let indexer_store = IndexerStore::new(indexer_db_path.clone())?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db_path)?;

    Ok((indexer_store, indexer_reader))
}

pub fn create_publish_transaction(
    test_transaction_builder: &TestTransactionBuilder,
    keystore: &InMemKeystore,
) -> Result<RoochTransaction> {
    let publish_action = test_transaction_builder.new_publish_examples(
        EXAMPLE_SIMPLE_BLOG_PACKAGE_NAME,
        Some(EXAMPLE_SIMPLE_BLOG_NAMED_ADDRESS.to_string()),
    )?;
    let tx_data = test_transaction_builder.build(publish_action);
    let rooch_tx =
        keystore.sign_transaction(&test_transaction_builder.sender.into(), tx_data, None)?;
    Ok(rooch_tx)
}

pub fn create_l2_tx(
    test_transaction_builder: &mut TestTransactionBuilder,
    keystore: &InMemKeystore,
    seq_num: u64,
) -> Result<RoochTransaction> {
    test_transaction_builder.update_sequence_number(seq_num);

    let action = match *TX_TYPE {
        TxType::Empty => test_transaction_builder.call_empty_create(),
        TxType::Transfer => test_transaction_builder.call_transfer_create(),
        TxType::Blog => test_transaction_builder.call_article_create_with_size(*TX_SIZE),
        _ => panic!("Unsupported tx type"),
    };

    let tx_data = test_transaction_builder.build(action);
    let rooch_tx =
        keystore.sign_transaction(&test_transaction_builder.sender.into(), tx_data, None)?;
    Ok(rooch_tx)
}

pub fn find_block_height() -> Result<Vec<u64>> {
    let dir = BTC_BLK_DIR.clone();

    let mut block_heights = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "hex" {
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let height: u64 = file_stem
                .parse()
                .expect("Failed to parse block height from filename");
            block_heights.push(height);
        }
    }

    block_heights.sort();
    Ok(block_heights)
}

pub fn create_btc_blk_tx(height: u64, block_file: String) -> Result<L1BlockWithBody> {
    let block_hex_str = fs::read_to_string(block_file).unwrap();
    let block_hex = Vec::<u8>::from_hex(&block_hex_str).unwrap();
    let origin_block: Block = deserialize(&block_hex).unwrap();
    let block = origin_block.clone();
    let block_hash = block.header.block_hash();
    let move_block = rooch_types::bitcoin::types::Block::try_from(block.clone()).unwrap();
    Ok(L1BlockWithBody {
        block: rooch_types::transaction::L1Block {
            chain_id: RoochMultiChainID::Bitcoin.multichain_id(),
            block_height: height,
            block_hash: block_hash.to_byte_array().to_vec(),
        },
        block_body: move_block.encode(),
    })
}

// pure execution, no validate, sequence
pub fn tx_exec_benchmark(c: &mut Criterion) {
    let mut binding_test = binding_test::RustBindingTest::new().unwrap();
    let keystore = InMemKeystore::new_insecure_for_tests(10);

    let default_account = keystore.addresses()[0];
    let mut test_transaction_builder = TestTransactionBuilder::new(default_account.into());

    let mut tx_cnt = 300;

    match *TX_TYPE {
        Blog => {
            let tx = create_publish_transaction(&test_transaction_builder, &keystore).unwrap();
            binding_test.execute(tx).unwrap();
        }
        BTCBlk => tx_cnt = 200,
        Transfer => tx_cnt = 500,
        Empty => tx_cnt = 1000,
    }

    let mut transactions: Vec<_> = Vec::with_capacity(tx_cnt);
    if *TX_TYPE != BTCBlk {
        for n in 0..tx_cnt {
            let tx = create_l2_tx(&mut test_transaction_builder, &keystore, n as u64).unwrap();
            transactions.push(binding_test.executor.validate_l2_tx(tx.clone()).unwrap());
        }
    } else {
        let heights = find_block_height().unwrap();
        let mut cnt = 0;
        for height in heights {
            if cnt >= tx_cnt {
                break;
            }
            let filename = format!("{}.hex", height);
            let file_path = [BTC_BLK_DIR.clone(), "/".parse().unwrap(), filename].concat();
            let l1_block = create_btc_blk_tx(height, file_path).unwrap();
            let ctx = binding_test.create_bt_blk_tx_ctx(cnt as u64, l1_block.clone());
            let move_tx = binding_test
                .executor
                .validate_l1_block(ctx, l1_block.clone())
                .unwrap();
            transactions.push(move_tx);
            cnt += 1;
        }
    }

    let mut transactions_iter = transactions.into_iter().cycle();

    c.bench_function("tx_exec", |b| {
        b.iter(|| {
            let tx = transactions_iter.next().unwrap();
            binding_test.execute_verified_tx(tx.clone()).unwrap()
        });
    });
}
