// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::scheduler::timer::Timer;
use coerce::actor::system::ActorSystem;
use coerce::actor::IntoActor;
use criterion::{criterion_group, criterion_main, Criterion};
use moveos_config::store_config::RocksdbConfig;
use moveos_config::{temp_dir, DataDirPath};
use moveos_store::{MoveOSDB, MoveOSStore};
// use pprof::criterion::{Output, PProfProfiler};
use raw_store::rocks::RocksDB;
use raw_store::StoreInstance;
use rooch_config::indexer_config::IndexerConfig;
use rooch_config::store_config::StoreConfig;
use rooch_da::actor::da::DAActor;
use rooch_da::proxy::DAProxy;
use rooch_da::server::serverproxy::DAServerNopProxy;
use rooch_da::server::serverproxy::DAServerProxy;
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::actor::indexer::IndexerActor;
use rooch_indexer::actor::reader_indexer::IndexerReaderActor;
use rooch_indexer::indexer_reader::IndexerReader;
use rooch_indexer::proxy::IndexerProxy;
use rooch_indexer::IndexerStore;
use rooch_key::key_derive::{generate_new_key_pair, retrieve_key_pair};
use rooch_proposer::actor::messages::ProposeBlock;
use rooch_proposer::actor::proposer::ProposerActor;
use rooch_proposer::proxy::ProposerProxy;
use rooch_rpc_server::service::aggregate_service::AggregateService;
use rooch_rpc_server::service::rpc_service::RpcService;
use rooch_sequencer::actor::sequencer::SequencerActor;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_store::RoochStore;
use rooch_types::address::RoochAddress;
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::bitcoin::network::Network;
use rooch_types::chain_id::RoochChainID;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::test_utils::random_typed_transaction_for_rooch_function;
use rooch_types::transaction::TypedTransaction;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tracing::info;

pub struct StoreHolder {
    _moveos_store: MoveOSStore,
    _rooch_store: RoochStore,
    _indexer_store: IndexerStore,
}
fn transaction_benchmark(c: &mut Criterion) {
    let tempdir = temp_dir();

    let rt: Runtime = Runtime::new().unwrap();
    let (rpc_service, _aggregate_service) =
        rt.block_on(async { init_service(&tempdir).await.unwrap() });

    let mut transactions = (0..500).cycle().map(|_v| create_transaction());
    c.bench_function("execute_tx", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| rpc_service.execute_tx(transactions.next().unwrap()))
    });
}

fn create_transaction() -> TypedTransaction {
    random_typed_transaction_for_rooch_function()
}

async fn init_service(datadir: &DataDirPath) -> Result<(RpcService, AggregateService)> {
    // We may call `start_server` multiple times in testing scenarios
    // tracing_subscriber can only be inited once.
    let _ = tracing_subscriber::fmt::try_init();

    let actor_system = ActorSystem::global_system();
    let chain_id = RoochChainID::LOCAL;

    // init storage
    let (moveos_store, rooch_store) = init_storage(datadir)?;
    let (indexer_store, indexer_reader) = init_indexer(datadir)?;

    // init key pair and accounts
    let key_pair = generate_new_key_pair(None, None, None, None)?;
    let rooch_key_pair: RoochKeyPair =
        retrieve_key_pair(&key_pair.key_pair_data.private_key_encryption, None)?;
    let rooch_account = RoochAddress::from(&rooch_key_pair.public());
    let sequencer_keypair = rooch_key_pair.copy();
    let proposer_keypair = rooch_key_pair.copy();
    let relayer_keypair = rooch_key_pair.copy();
    let sequencer_account = RoochAddress::from(&sequencer_keypair.public());
    let proposer_account = RoochAddress::from(&proposer_keypair.public());
    let _relayer_account = RoochAddress::from(&relayer_keypair.public());

    // Init executor
    let is_genesis = moveos_store.statedb.is_genesis();
    let btc_network = Network::default().to_num();
    let executor_actor = ExecutorActor::new(
        chain_id.genesis_ctx(rooch_account),
        BitcoinGenesisContext::new(btc_network),
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
    let da_server_proxies: Vec<Arc<dyn DAServerProxy + Send + Sync>> =
        vec![Arc::new(DAServerNopProxy {})];
    let da_proxy = DAProxy::new(
        DAActor::new(da_server_proxies)
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

    let rpc_service = RpcService::new(
        chain_id.chain_id().id(),
        executor_proxy.clone(),
        sequencer_proxy,
        proposer_proxy,
        indexer_proxy,
    );
    let aggregate_service = AggregateService::new(rpc_service.clone());

    Ok((rpc_service, aggregate_service))
}

fn init_storage(datadir: &DataDirPath) -> Result<(MoveOSStore, RoochStore)> {
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

fn init_indexer(datadir: &DataDirPath) -> Result<(IndexerStore, IndexerReader)> {
    let indexer_db_path = IndexerConfig::get_mock_indexer_db(datadir);
    let indexer_db_parent_dir = indexer_db_path
        .parent()
        .ok_or(anyhow::anyhow!("Invalid indexer db dir"))?;
    if !indexer_db_parent_dir.exists() {
        std::fs::create_dir_all(indexer_db_parent_dir)?;
    }
    if !indexer_db_path.exists() {
        std::fs::File::create(indexer_db_path.clone())?;
    };
    let indexer_db_url = indexer_db_path
        .to_str()
        .ok_or(anyhow::anyhow!("Invalid indexer db path"))?;
    let indexer_store = IndexerStore::new(indexer_db_url)?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db_url)?;

    Ok((indexer_store, indexer_reader))
}

criterion_group! {
    name = rooch_transaction_benches;
    config = Criterion::default().sample_size(200).measurement_time(Duration::from_secs(10));
    // config = Criterion::default().sample_size(200).measurement_time(Duration::from_secs(10))
    // .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = transaction_benchmark
}
criterion_main!(rooch_transaction_benches);
