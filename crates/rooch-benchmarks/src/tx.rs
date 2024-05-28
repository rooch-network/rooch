// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::time::Duration;

use anyhow::Result;
use bitcoin::consensus::deserialize;
use bitcoin::hashes::Hash;
use bitcoin::hex::FromHex;
use bitcoincore_rpc_json::bitcoin;
use bitcoincore_rpc_json::bitcoin::Block;
use coerce::actor::scheduler::timer::Timer;
use coerce::actor::system::ActorSystem;
use coerce::actor::IntoActor;
use tracing::info;

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
use rooch_genesis::RoochGenesis;
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
use rooch_types::crypto::RoochKeyPair;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::rooch_network::{BuiltinChainID, RoochNetwork};
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::transaction::L1BlockWithBody;

use crate::config::TxType;
use crate::tx::TxType::{Empty, Transfer};

pub const EXAMPLE_SIMPLE_BLOG_PACKAGE_NAME: &str = "simple_blog";
pub const EXAMPLE_SIMPLE_BLOG_NAMED_ADDRESS: &str = "simple_blog";

pub fn gen_sequencer(keypair: RoochKeyPair, rooch_store: RoochStore) -> Result<SequencerActor> {
    SequencerActor::new(keypair, rooch_store.clone())
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

    // init storage
    let (mut moveos_store, mut rooch_store) = init_storage(datadir)?;
    let (indexer_store, indexer_reader) = init_indexer(datadir)?;

    // init keystore
    let rooch_account = keystore.addresses()[0];
    let rooch_key_pair = keystore
        .get_key_pairs(&rooch_account, None)?
        .pop()
        .expect("Key pair should have value");

    let sequencer_keypair = rooch_key_pair.copy();
    let proposer_keypair = rooch_key_pair.copy();
    let sequencer_account = RoochAddress::from(&sequencer_keypair.public());
    let proposer_account = RoochAddress::from(&proposer_keypair.public());

    // Init executor
    let mut network: RoochNetwork = BuiltinChainID::Dev.into();
    network.set_sequencer_account(rooch_account.into());
    let genesis: RoochGenesis = RoochGenesis::build(network)?;
    let root = genesis.init_genesis(&mut moveos_store, &mut rooch_store)?;

    let executor_actor =
        ExecutorActor::new(root.clone(), moveos_store.clone(), rooch_store.clone())?;
    let reader_executor =
        ReaderExecutorActor::new(root.clone(), moveos_store.clone(), rooch_store.clone())?
            .into_actor(Some("ReaderExecutor"), &actor_system)
            .await?;
    let executor = executor_actor
        .into_actor(Some("Executor"), &actor_system)
        .await?;
    let executor_proxy = ExecutorProxy::new(executor.into(), reader_executor.into());

    // Init sequencer
    info!("RPC Server sequencer address: {:?}", sequencer_account);
    let sequencer = SequencerActor::new(sequencer_keypair, rooch_store.clone())?
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
    let indexer_executor = IndexerActor::new(root, indexer_store.clone(), moveos_store.clone())?
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
        fs::create_dir_all(rooch_db_path.clone())?;
    }
    if !moveos_db_path.exists() {
        fs::create_dir_all(moveos_db_path.clone())?;
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
    tx_type: TxType,
) -> Result<RoochTransaction> {
    test_transaction_builder.update_sequence_number(seq_num);

    let action = match tx_type {
        Empty => test_transaction_builder.call_empty_create(),
        Transfer => test_transaction_builder.call_transfer_create(),
        _ => panic!("Unsupported tx type"),
    };

    let tx_data = test_transaction_builder.build(action);
    let rooch_tx =
        keystore.sign_transaction(&test_transaction_builder.sender.into(), tx_data, None)?;
    Ok(rooch_tx)
}

pub fn find_block_height(dir: String) -> Result<Vec<u64>> {
    let mut block_heights = Vec::new();

    for entry in fs::read_dir(dir)? {
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
    let move_block = rooch_types::bitcoin::types::Block::from(block.clone());
    Ok(L1BlockWithBody {
        block: rooch_types::transaction::L1Block {
            chain_id: RoochMultiChainID::Bitcoin.multichain_id(),
            block_height: height,
            block_hash: block_hash.to_byte_array().to_vec(),
        },
        block_body: move_block.encode(),
    })
}
