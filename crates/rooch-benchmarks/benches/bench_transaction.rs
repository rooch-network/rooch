// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use rooch_benchmarks::transaction::TransactionBencher;
// #[allow(deprecated)]
// use criterion::{criterion_group, criterion_main, Benchmark, Criterion};
use criterion::{criterion_group, criterion_main, Criterion};
// #[cfg(target_os = "linux")]
use pprof::criterion::{Output, PProfProfiler};
use std::time::Duration;
use coerce::actor::IntoActor;
use coerce::actor::scheduler::timer::Timer;
use coerce::actor::system::ActorSystem;
use tokio::runtime::Runtime;
use tracing::info;
use moveos_store::MoveOSStore;
use rooch_config::da_config::DAConfig;
use rooch_da::actor::da::DAActor;
use rooch_da::proxy::DAProxy;
use rooch_da::server::serverproxy::DAServerProxy;
use rooch_da::server::serverproxy::DAServerNopProxy;
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::actor::indexer::IndexerActor;
use rooch_indexer::actor::reader_indexer::IndexerReaderActor;
use rooch_indexer::indexer_reader::IndexerReader;
use rooch_indexer::IndexerStore;
use rooch_indexer::proxy::IndexerProxy;
use rooch_key::key_derive::{generate_new_key_pair, retrieve_key_pair};
use rooch_proposer::actor::messages::ProposeBlock;
use rooch_proposer::actor::proposer::ProposerActor;
use rooch_proposer::proxy::ProposerProxy;
use rooch_rpc_server::service::aggregate_service::AggregateService;
use rooch_sequencer::actor::sequencer::SequencerActor;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_store::RoochStore;
use rooch_types::address::{RoochAddress, RoochSupportedAddress};
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::bitcoin::network::Network;
use rooch_types::chain_id::RoochChainID;
use rooch_types::crypto::RoochKeyPair;

// #[allow(deprecated)]
// fn block_apply(c: &mut Criterion) {
//     ::starcoin_logger::init();
//     for i in &[10u64, 1000] {
//         c.bench(
//             "block_apply",
//             Benchmark::new(format!("block_apply_{:?}", i), move |b| {
//                 let bencher = BlockBencher::new(Some(*i));
//                 bencher.bench(b)
//             })
//             .sample_size(10),
//         );
//     }
// }
//
// #[allow(deprecated)]
// fn query_block(c: &mut Criterion) {
//     ::starcoin_logger::init();
//     for block_num in &[10u64, 1000u64] {
//         let bencher = BlockBencher::new(Some(*block_num));
//         bencher.execute();
//
//         for i in &[100u64, 1000, 10000] {
//             let id = format!("query_block_in({:?})_times({:?})", block_num, i,);
//             let bencher_local = bencher.clone();
//             c.bench(
//                 "query_block",
//                 Benchmark::new(id, move |b| bencher_local.query_bench(b, *i)).sample_size(10),
//             );
//         }
//     }
// }
// #[cfg(target_os = "linux")]
// criterion_group!(
//     name=rooch_block_benches;
//     config = Criterion::default()
//     .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
//     targets=block_apply,query_block);
// #[cfg(not(target_os = "linux"))]
// criterion_group!(rooch_block_benches, block_apply, query_block);
// criterion_main!(rooch_block_benches);

fn transaction_benchmark(c: &mut Criterion) {
    // let pg_host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into());
    // let pg_port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "32770".into());
    // let pw = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgrespw".into());
    // let db_url = format!("postgres://postgres:{pw}@{pg_host}:{pg_port}");

    let rt: Runtime = Runtime::new().unwrap();
    let (mut _checkpoints, store) = rt.block_on(async {
        let blocking_cp = new_pg_connection_pool(&db_url).unwrap();
        reset_database(&mut blocking_cp.get().unwrap(), true, false).unwrap();
        let registry = Registry::default();
        let indexer_metrics = IndexerMetrics::new(&registry);

        let store = PgIndexerStore::new(blocking_cp, indexer_metrics);

        let checkpoints = (0..150).map(create_checkpoint).collect::<Vec<_>>();
        (checkpoints, store)
    });

    // TODO(gegaowp): add updated data ingestion benchmarking steps here.
    let mut checkpoints = (20..100).cycle().map(CheckpointId::SequenceNumber);
    c.bench_function("get_checkpoint", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| store.get_checkpoint(checkpoints.next().unwrap()))
    });
}

fn create_checkpoint(sequence_number: i64) -> TemporaryCheckpointStore {
    TemporaryCheckpointStore {
        checkpoint: Checkpoint {
            sequence_number,
            checkpoint_digest: CheckpointDigest::random().base58_encode(),
            epoch: 0,
            transactions: vec![],
            previous_checkpoint_digest: Some(CheckpointDigest::random().base58_encode()),
            end_of_epoch: false,
            validator_signature: AggregateAuthoritySignature::default().to_string(),
            total_gas_cost: i64::MAX,
            total_computation_cost: i64::MAX,
            total_storage_cost: i64::MAX,
            total_storage_rebate: i64::MAX,
            total_transaction_blocks: 1000,
            total_transactions: 1000,
            total_successful_transaction_blocks: 1000,
            total_successful_transactions: 1000,
            network_total_transactions: 0,
            timestamp_ms: Utc::now().timestamp_millis(),
        },
        transactions: (1..1000)
            .map(|_| create_transaction(sequence_number))
            .collect(),
        events: vec![],
        input_objects: vec![],
        changed_objects: vec![],
        move_calls: vec![],
        recipients: vec![],
    }
}

fn create_transaction(sequence_number: i64) -> Transaction {
    let gas_price = 1000;
    let tx = TransactionData::new_pay_sui(
        SuiAddress::random_for_testing_only(),
        vec![],
        vec![SuiAddress::random_for_testing_only()],
        vec![100000],
        (
            ObjectID::random(),
            SequenceNumber::new(),
            ObjectDigest::random(),
        ),
        gas_price * TEST_ONLY_GAS_UNIT_FOR_TRANSFER,
        gas_price,
    )
        .unwrap();

    Transaction {
        id: None,
        transaction_digest: TransactionDigest::random().base58_encode(),
        sender: SuiAddress::random_for_testing_only().to_string(),
        checkpoint_sequence_number: Some(sequence_number),
        timestamp_ms: Some(Utc::now().timestamp_millis()),
        transaction_kind: "test".to_string(),
        transaction_count: 0,
        execution_success: true,
        gas_object_id: ObjectID::random().to_string(),
        gas_object_sequence: 0,
        gas_object_digest: ObjectDigest::random().base58_encode(),
        gas_budget: 0,
        total_gas_cost: 0,
        computation_cost: 0,
        storage_cost: 0,
        storage_rebate: 0,
        non_refundable_storage_fee: 0,
        gas_price: 0,
        raw_transaction: bcs::to_bytes(&tx).unwrap(),
        transaction_effects_content: "".to_string(),
        confirmed_local_execution: None,
    }
}



fn init_service() -> Result<RpcService, AggregateService> {
    // // We may call `start_server` multiple times in testing scenarios
    // // tracing_subscriber can only be inited once.
    // let _ = tracing_subscriber::fmt::try_init();
    //
    // let config = opt.port.map_or(ServerConfig::default(), |port| {
    //     ServerConfig::new_with_port(port)
    // });
    // let chain_id_opt = opt.chain_id.clone().unwrap_or_default();
    //
    // let actor_system = ActorSystem::global_system();
    //
    // //Init store
    // let base_config = BaseConfig::load_with_opt(opt)?;
    // let mut store_config = StoreConfig::default();
    // store_config.merge_with_opt_with_init(opt, Arc::new(base_config.clone()), true)?;
    // let (moveos_store, rooch_store) = init_storage(&store_config)?;
    //
    // //Init indexer store
    // let mut indexer_config = IndexerConfig::default();
    // indexer_config.merge_with_opt_with_init(opt, Arc::new(base_config), true)?;
    // let (indexer_store, indexer_reader) = init_indexer(&indexer_config)?;
    //
    // // Check for key pairs
    // if server_opt.sequencer_keypair.is_none()
    //     || server_opt.proposer_keypair.is_none()
    //     || server_opt.relayer_keypair.is_none()
    // {
    //     // only for integration test, generate test key pairs
    //     if chain_id_opt.is_test_or_dev_or_local() {
    //         let result = generate_new_key_pair(None, None, None, None)?;
    //         let kp: RoochKeyPair =
    //             retrieve_key_pair(&result.key_pair_data.private_key_encryption, None)?;
    //         server_opt.sequencer_keypair = Some(kp.copy());
    //         server_opt.proposer_keypair = Some(kp.copy());
    //         server_opt.relayer_keypair = Some(kp.copy());
    //     } else {
    //         return Err(Error::from(
    //             RoochError::InvalidSequencerOrProposerOrRelayerKeyPair,
    //         ));
    //     }
    // }


    // We may call `start_server` multiple times in testing scenarios
    // tracing_subscriber can only be inited once.
    let _ = tracing_subscriber::fmt::try_init();

    let actor_system = ActorSystem::global_system();
    // let chain_id_opt = opt.chain_id.clone().unwrap_or_default();
    let chain_id = RoochChainID::LOCAL;

    // init storage
    let moveos_store = MoveOSStore::mock_moveos_store()?;
    let rooch_store = RoochStore::mock_rooch_store()?;
    // let rooch_account = RoochAddress::random();
    let indexer_db_url = IndexerStore::mock_db_url()?.as_str();
    let indexer_store = IndexerStore::new(&indexer_db_url)?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(&indexer_db_url)?;


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
    let relayer_account = RoochAddress::from(&relayer_keypair.public());

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
        .into_actor(Some("ReaderExecutor"), &actor_system)?;
    let executor = executor_actor
        .into_actor(Some("Executor"), &actor_system)?;
    let executor_proxy = ExecutorProxy::new(executor.into(), reader_executor.into());

    // Init sequencer
    info!("RPC Server sequencer address: {:?}", sequencer_account);
    let sequencer = SequencerActor::new(sequencer_keypair, rooch_store, is_genesis)?
        .into_actor(Some("Sequencer"), &actor_system)?;
    let sequencer_proxy = SequencerProxy::new(sequencer.into());

    // Init DA
    // let mut da_config = DAConfig::default(); // TODO use opt
    // da_config.merge_with_opt(opt)?;
    let da_server_proxies: Vec<Arc<dyn DAServerProxy + Send + Sync>> = vec![Arc::new(DAServerNopProxy {})];

    // if let Some(internal_da_server_config) = &da_config.internal_da_server {
    //     for server_config_type in &internal_da_server_config.servers {
    //         if let InternalDAServerConfigType::Celestia(celestia_config) = server_config_type {
    //             let da_server = DAServerCelestiaActor::new(celestia_config)
    //                 .await
    //                 .into_actor(Some("DAServerCelestia"), &actor_system)
    //                 .await?;
    //             da_server_proxies.push(Arc::new(DAServerCelestiaProxy::new(
    //                 da_server.clone().into(),
    //             )));
    //         }
    //     }
    // } else {
    //     da_server_proxies.push(Arc::new(DAServerNopProxy {}));
    // }
    let da_proxy = DAProxy::new(
        DAActor::new(da_server_proxies)
            .into_actor(Some("DAProxy"), &actor_system)?
            .into(),
    );

    // Init proposer
    // let proposer_keypair = server_opt.proposer_keypair.unwrap();
    // let proposer_account: RoochAddress = (&proposer_keypair.public()).into();
    info!("RPC Server proposer address: {:?}", proposer_account);
    let proposer = ProposerActor::new(proposer_keypair, da_proxy)
        .into_actor(Some("Proposer"), &actor_system)?;
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
    let indexer_executor = IndexerActor::new(indexer_store, moveos_store)?
        .into_actor(Some("Indexer"), &actor_system)?;
    let indexer_reader_executor = IndexerReaderActor::new(indexer_reader)?
        .into_actor(Some("IndexerReader"), &actor_system)?;
    let indexer_proxy = IndexerProxy::new(indexer_executor.into(), indexer_reader_executor.into());

    let rpc_service = RpcService::new(
        chain_id.chain_id().id(),
        executor_proxy.clone(),
        sequencer_proxy,
        proposer_proxy,
        indexer_proxy,
    );
    let aggregate_service = AggregateService::new(rpc_service.clone());

    // let ethereum_relayer_config = opt.ethereum_relayer_config();
    // let bitcoin_relayer_config = opt.bitcoin_relayer_config();
    //
    // if ethereum_relayer_config.is_some() || bitcoin_relayer_config.is_some() {
    //     let relayer_keypair = server_opt.relayer_keypair.unwrap();
    //     let relayer_account: RoochAddress = (&relayer_keypair.public()).into();
    //     info!("RPC Server relayer address: {:?}", relayer_account);
    //     let relayer = RelayerActor::new(
    //         executor_proxy,
    //         relayer_keypair,
    //         ethereum_relayer_config,
    //         bitcoin_relayer_config,
    //         rpc_service.clone(),
    //     )
    //         .await?
    //         .into_actor(Some("Relayer"), &actor_system)
    //         .await?;
    //     let relay_tick_in_seconds: u64 = 1;
    //     let relayer_timer = Timer::start(
    //         relayer,
    //         Duration::from_secs(relay_tick_in_seconds),
    //         RelayTick {},
    //     );
    //     timers.push(relayer_timer);
    // }
    //
    // let acl = match env::var("ACCESS_CONTROL_ALLOW_ORIGIN") {
    //     Ok(value) => {
    //         let allow_hosts = value
    //             .split(',')
    //             .map(HeaderValue::from_str)
    //             .collect::<Result<Vec<_>, _>>()?;
    //         AllowOrigin::list(allow_hosts)
    //     }
    //     _ => AllowOrigin::any(),
    // };
    // info!(?acl);
    //
    // let cors: CorsLayer = CorsLayer::new()
    //     // Allow `POST` when accessing the resource
    //     .allow_methods([Method::POST])
    //     // Allow requests from any origin
    //     .allow_origin(acl)
    //     .allow_headers([hyper::header::CONTENT_TYPE]);
    //
    // let middleware = tower::ServiceBuilder::new()
    //     .layer(TraceLayer::new_for_http())
    //     .layer(cors);
    //
    // // Build server
    // let server = ServerBuilder::default()
    //     .set_logger(RpcLogger)
    //     .set_middleware(middleware)
    //     .build(&addr)
    //     .await?;
    //
    // let mut rpc_module_builder = RpcModuleBuilder::new();
    // rpc_module_builder.register_module(RoochServer::new(
    //     rpc_service.clone(),
    //     aggregate_service.clone(),
    // ))?;
    // rpc_module_builder.register_module(EthNetServer::new(chain_id_opt.chain_id()))?;
    // rpc_module_builder.register_module(EthServer::new(
    //     chain_id_opt.chain_id(),
    //     rpc_service.clone(),
    //     aggregate_service.clone(),
    // ))?;
    // rpc_module_builder.register_module(BtcServer::new(
    //     rpc_service.clone(),
    //     aggregate_service.clone(),
    //     btc_network,
    // ))?;
    //
    // // let rpc_api = build_rpc_api(rpc_api);
    // let methods_names = rpc_module_builder.module.method_names().collect::<Vec<_>>();
    // let handle = server.start(rpc_module_builder.module)?;

    // info!("JSON-RPC HTTP Server start listening {:?}", addr);
    // info!("Available JSON-RPC methods : {:?}", methods_names);

    Ok((rpc_service, aggregate_service))
}





criterion_group! {
    name = rooch_transaction_benches;
    config = Criterion::default().sample_size(50).measurement_time(Duration::from_secs(10));
    targets = transaction_benchmark
}
criterion_main!(rooch_transaction_benches);

// #[cfg(target_os = "linux")]
// criterion_group!(
//     name=rooch_block_benches;
//     config = Criterion::default()
//     .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
//     targets=block_apply,query_block);
// #[cfg(not(target_os = "linux"))]
// criterion_group!(rooch_block_benches, block_apply, query_block);
// criterion_main!(rooch_block_benches);