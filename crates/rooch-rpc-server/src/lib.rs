// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::metrics_server::{init_metrics, start_basic_prometheus_server};
use crate::server::btc_server::BtcServer;
use crate::server::rooch_server::RoochServer;
use crate::service::aggregate_service::AggregateService;
use crate::service::rpc_logger::RpcLogger;
use crate::service::rpc_service::RpcService;
use anyhow::{ensure, Error, Result};
use axum::http::{HeaderValue, Method};
use coerce::actor::scheduler::timer::Timer;
use coerce::actor::{system::ActorSystem, IntoActor};
use jsonrpsee::server::middleware::rpc::RpcServiceBuilder;
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::RpcModule;
use moveos_eventbus::bus::EventBus;
use raw_store::errors::RawStoreError;
use rooch_config::server_config::ServerConfig;
use rooch_config::{RoochOpt, ServerOpt};
use rooch_da::actor::da::DAActor;
use rooch_da::proxy::DAProxy;
use rooch_db::RoochDB;
use rooch_event::actor::EventActor;
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::actor::reader_executor::ReaderExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_genesis::RoochGenesis;
use rooch_indexer::actor::indexer::IndexerActor;
use rooch_indexer::actor::reader_indexer::IndexerReaderActor;
use rooch_indexer::proxy::IndexerProxy;
use rooch_pipeline_processor::actor::processor::PipelineProcessorActor;
use rooch_pipeline_processor::proxy::PipelineProcessorProxy;
use rooch_proposer::actor::messages::ProposeBlock;
use rooch_proposer::actor::proposer::ProposerActor;
use rooch_proposer::proxy::ProposerProxy;
use rooch_relayer::actor::bitcoin_client::BitcoinClientActor;
use rooch_relayer::actor::bitcoin_client_proxy::BitcoinClientProxy;
use rooch_relayer::actor::messages::RelayTick;
use rooch_relayer::actor::relayer::RelayerActor;
use rooch_rpc_api::api::RoochRpcModule;
use rooch_rpc_api::RpcError;
use rooch_sequencer::actor::sequencer::SequencerActor;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::address::RoochAddress;
use rooch_types::error::{GenesisError, RoochError};
use rooch_types::rooch_network::BuiltinChainID;
use serde_json::json;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::Duration;
use std::{env, panic, process};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

pub mod metrics_server;
pub mod server;
pub mod service;

/// This exit code means is that the server failed to start and required human intervention.
static R_EXIT_CODE_NEED_HELP: i32 = 120;

pub struct ServerHandle {
    handle: jsonrpsee::server::ServerHandle,
    timers: Vec<Timer>,
    _opt: RoochOpt,
    _prometheus_registry: prometheus::Registry,
}

impl ServerHandle {
    fn stop(self) -> Result<()> {
        for timer in self.timers {
            timer.stop();
        }
        self.handle.stop()?;
        Ok(())
    }
}

impl Debug for ServerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerHandle")
            .field("handle", &self.handle)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct Service {
    handle: Option<ServerHandle>,
}

impl Service {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub async fn start(&mut self, opt: RoochOpt, server_opt: ServerOpt) -> Result<()> {
        self.handle = Some(start_server(opt, server_opt).await?);
        Ok(())
    }

    pub fn stop(self) -> Result<()> {
        if let Some(handle) = self.handle {
            handle.stop()?
        }
        Ok(())
    }
}

pub struct RpcModuleBuilder {
    module: RpcModule<()>,
}

impl Default for RpcModuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcModuleBuilder {
    pub fn new() -> Self {
        Self {
            module: RpcModule::new(()),
        }
    }

    pub fn register_module<M: RoochRpcModule>(&mut self, module: M) -> Result<()> {
        Ok(self.module.merge(module.rpc())?)
    }
}

// Start json-rpc server
pub async fn start_server(opt: RoochOpt, server_opt: ServerOpt) -> Result<ServerHandle> {
    let chain_name = opt.chain_id().chain_name();
    match run_start_server(opt, server_opt).await {
        Ok(server_handle) => Ok(server_handle),
        Err(e) => match e.downcast::<GenesisError>() {
            Ok(e) => {
                log::error!(
                    "{:?}, please clean your data dir. `rooch server clean -n {}` ",
                    e,
                    chain_name
                );
                std::process::exit(R_EXIT_CODE_NEED_HELP);
            }
            Err(e) => match e.downcast::<RawStoreError>() {
                Ok(e) => {
                    log::error!(
                        "{:?}, please clean your data dir. `rooch server clean -n {}` ",
                        e,
                        chain_name
                    );
                    std::process::exit(R_EXIT_CODE_NEED_HELP);
                }
                Err(e) => {
                    log::error!("{:?}, server start fail. ", e);
                    std::process::exit(R_EXIT_CODE_NEED_HELP);
                }
            },
        },
    }
}

// run json-rpc server
pub async fn run_start_server(opt: RoochOpt, server_opt: ServerOpt) -> Result<ServerHandle> {
    // We may call `start_server` multiple times in testing scenarios
    // tracing_subscriber can only be inited once.
    let _ = tracing_subscriber::fmt::try_init();

    //Exit the process when some thread panic
    // take_hook() returns the default hook in case when a custom one is not set
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        error!("Panic occurred:\n {} \n exit the process", panic_info);
        process::exit(1);
    }));

    let config = ServerConfig::new_with_port(opt.port());
    let actor_system = ActorSystem::global_system();

    // start prometheus server
    let prometheus_registry = start_basic_prometheus_server();
    // Initialize metrics before creating any stores
    init_metrics(&prometheus_registry);

    //Init store
    let store_config = opt.store_config();

    let rooch_db = RoochDB::init(store_config, &prometheus_registry)?;
    let (rooch_store, moveos_store, indexer_store, indexer_reader) = (
        rooch_db.rooch_store.clone(),
        rooch_db.moveos_store.clone(),
        rooch_db.indexer_store.clone(),
        rooch_db.indexer_reader.clone(),
    );

    // Check for key pairs
    if server_opt.sequencer_keypair.is_none() || server_opt.proposer_keypair.is_none() {
        return Err(Error::from(
            RoochError::InvalidSequencerOrProposerOrRelayerKeyPair,
        ));
    }

    let sequencer_keypair = server_opt.sequencer_keypair.unwrap();
    let sequencer_account = sequencer_keypair.public().rooch_address()?;
    let sequencer_bitcoin_address = sequencer_keypair.public().bitcoin_address()?;

    let service_status = opt.service_status;

    let mut network = opt.network();
    if network.chain_id == BuiltinChainID::Local.chain_id() {
        // local chain use current active account as sequencer account
        network.set_sequencer_account(sequencer_bitcoin_address);
    } else {
        ensure!(
            network.genesis_config.sequencer_account == sequencer_bitcoin_address,
            "Sequencer({:?}) in genesis config is not equal to sequencer({:?}) in cli config",
            network.genesis_config.sequencer_account,
            sequencer_bitcoin_address
        );
    }

    let genesis = RoochGenesis::load_or_init(network.clone(), &rooch_db)?;

    let root = match rooch_db.latest_root()? {
        Some(root) => root,
        None => genesis.genesis_root().clone(),
    };
    info!(
        "The latest Root object state root: {:?}, size: {}",
        root.state_root(),
        root.size()
    );

    let event_bus = EventBus::new();
    let event_actor = EventActor::new(event_bus.clone());
    let event_actor_ref = event_actor
        .into_actor(Some("EventActor"), &actor_system)
        .await?;

    let executor_actor = ExecutorActor::new(
        root.clone(),
        moveos_store.clone(),
        rooch_store.clone(),
        &prometheus_registry,
        Some(event_actor_ref.clone()),
    )?;

    let executor_actor_ref = executor_actor
        .into_actor(Some("Executor"), &actor_system)
        .await?;

    let reader_executor = ReaderExecutorActor::new(
        root.clone(),
        moveos_store.clone(),
        rooch_store.clone(),
        Some(event_actor_ref),
    )?;

    let read_executor_ref = reader_executor
        .into_actor(Some("ReadExecutor"), &actor_system)
        .await?;

    let executor_proxy = ExecutorProxy::new(
        executor_actor_ref.clone().into(),
        read_executor_ref.clone().into(),
    );

    // Init sequencer
    info!("RPC Server sequencer address: {:?}", sequencer_account);
    let sequencer = SequencerActor::new(
        sequencer_keypair.copy(),
        rooch_store,
        service_status,
        &prometheus_registry,
    )?
    .into_actor(Some("Sequencer"), &actor_system)
    .await?;
    let sequencer_proxy = SequencerProxy::new(sequencer.into());

    // Init DA
    let da_config = opt.da_config().clone();

    let da_proxy = DAProxy::new(
        DAActor::new(da_config, &actor_system)
            .await?
            .into_actor(Some("DAProxy"), &actor_system)
            .await?
            .into(),
    );

    // Init proposer
    let proposer_keypair = server_opt.proposer_keypair.unwrap();
    let proposer_account: RoochAddress = proposer_keypair.public().rooch_address()?;
    info!("RPC Server proposer address: {:?}", proposer_account);
    let proposer = ProposerActor::new(proposer_keypair, da_proxy, &prometheus_registry)
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
    let indexer_executor = IndexerActor::new(root, indexer_store)?
        .into_actor(Some("Indexer"), &actor_system)
        .await?;
    let indexer_reader_executor = IndexerReaderActor::new(indexer_reader)?
        .into_actor(Some("IndexerReader"), &actor_system)
        .await?;
    let indexer_proxy = IndexerProxy::new(indexer_executor.into(), indexer_reader_executor.into());

    let mut processor = PipelineProcessorActor::new(
        executor_proxy.clone(),
        sequencer_proxy.clone(),
        proposer_proxy.clone(),
        indexer_proxy.clone(),
        service_status,
        &prometheus_registry,
    );

    // Only process sequenced tx on startup when service is active
    if service_status.is_active() {
        processor.process_sequenced_tx_on_startup().await?;
    }
    let processor_actor = processor
        .into_actor(Some("PipelineProcessor"), &actor_system)
        .await?;
    let processor_proxy = PipelineProcessorProxy::new(processor_actor.into());

    let ethereum_relayer_config = opt.ethereum_relayer_config();
    let bitcoin_relayer_config = opt.bitcoin_relayer_config();

    if service_status.is_active()
        && (ethereum_relayer_config.is_some() || bitcoin_relayer_config.is_some())
    {
        let relayer = RelayerActor::new(
            executor_proxy.clone(),
            processor_proxy.clone(),
            ethereum_relayer_config,
            bitcoin_relayer_config.clone(),
        )
        .await?
        .into_actor(Some("Relayer"), &actor_system)
        .await?;
        let relay_tick_in_seconds: u64 = 1;
        let relayer_timer = Timer::start(
            relayer,
            Duration::from_secs(relay_tick_in_seconds),
            RelayTick {},
        );
        timers.push(relayer_timer);
    }

    let bitcoin_client_proxy = if service_status.is_active() && bitcoin_relayer_config.is_some() {
        let bitcoin_config = bitcoin_relayer_config.unwrap();
        let bitcoin_client = BitcoinClientActor::new(
            &bitcoin_config.btc_rpc_url,
            &bitcoin_config.btc_rpc_user_name,
            &bitcoin_config.btc_rpc_password,
        )?;
        let bitcoin_client_actor_ref = bitcoin_client
            .into_actor(Some("bitcoin_client_for_rpc_service"), &actor_system)
            .await?;
        let bitcoin_client_proxy = BitcoinClientProxy::new(bitcoin_client_actor_ref.into());
        Some(bitcoin_client_proxy)
    } else {
        None
    };

    let rpc_service = RpcService::new(
        network.chain_id.id,
        network.genesis_config.bitcoin_network,
        executor_proxy,
        sequencer_proxy,
        indexer_proxy,
        processor_proxy,
        bitcoin_client_proxy,
    );
    let aggregate_service = AggregateService::new(rpc_service.clone());

    let acl = match env::var("ACCESS_CONTROL_ALLOW_ORIGIN") {
        Ok(value) => {
            let allow_hosts = value
                .split(',')
                .map(HeaderValue::from_str)
                .collect::<Result<Vec<_>, _>>()?;
            AllowOrigin::list(allow_hosts)
        }
        _ => AllowOrigin::any(),
    };
    info!(?acl);

    let cors: CorsLayer = CorsLayer::new()
        // Allow `POST` when accessing the resource
        .allow_methods([Method::POST])
        // Allow requests from any origin
        .allow_origin(acl)
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let middleware = tower::ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    let rpc_middleware = RpcServiceBuilder::new().layer_fn(RpcLogger);

    // Build server
    let server = ServerBuilder::default()
        .set_http_middleware(middleware)
        .set_rpc_middleware(rpc_middleware)
        .build(&addr)
        .await?;

    let mut rpc_module_builder = RpcModuleBuilder::new();
    rpc_module_builder.register_module(RoochServer::new(
        rpc_service.clone(),
        aggregate_service.clone(),
    ))?;
    rpc_module_builder.register_module(BtcServer::new(rpc_service.clone()).await?)?;
    rpc_module_builder
        .module
        .register_method("rpc.discover", move |_, _, _| {
            Ok::<rooch_open_rpc::Project, RpcError>(
                rooch_open_rpc_spec_builder::build_rooch_rpc_spec(),
            )
        })?;

    // let rpc_api = build_rpc_api(rpc_api);
    let methods_names = rpc_module_builder.module.method_names().collect::<Vec<_>>();
    let handle = server.start(rpc_module_builder.module);

    info!("JSON-RPC HTTP Server start listening {:?}", addr);
    info!("Available JSON-RPC methods : {:?}", methods_names);

    Ok(ServerHandle {
        handle,
        timers,
        _opt: opt,
        _prometheus_registry: prometheus_registry,
    })
}

fn _build_rpc_api<M: Send + Sync + 'static>(mut rpc_module: RpcModule<M>) -> RpcModule<M> {
    let mut available_methods = rpc_module.method_names().collect::<Vec<_>>();
    available_methods.sort();

    rpc_module
        .register_method("rpc_methods", move |_, _, _| {
            Ok::<serde_json::Value, RpcError>(json!({
                "methods": available_methods,
            }))
        })
        .expect("infallible all other methods have their own address space");

    rpc_module
}
