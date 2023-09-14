// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::server::eth_server::EthServer;
use crate::server::rooch_server::RoochServer;
use crate::server::wallet_server::WalletServer;
use crate::service::aggregate_service::AggregateService;
use crate::service::rpc_service::RpcService;
use anyhow::Result;
use coerce::actor::scheduler::timer::Timer;
use coerce::actor::{system::ActorSystem, IntoActor};
use hyper::header::HeaderValue;
use hyper::Method;
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::RpcModule;
use moveos_store::{MoveOSDB, MoveOSStore};
use raw_store::errors::RawStoreError;
use raw_store::rocks::RocksDB;
use raw_store::StoreInstance;
use rooch_config::server_config::ServerConfig;
use rooch_config::store_config::StoreConfig;
use rooch_config::{BaseConfig, RoochOpt};
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_key::key_derive::generate_new_key_pair;
use rooch_proposer::actor::messages::ProposeBlock;
use rooch_proposer::actor::proposer::ProposerActor;
use rooch_proposer::proxy::ProposerProxy;
use rooch_relayer::actor::messages::RelayTick;
use rooch_relayer::actor::relayer::RelayerActor;
use rooch_rpc_api::api::RoochRpcModule;
use rooch_sequencer::actor::sequencer::SequencerActor;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_store::RoochStore;
use rooch_types::address::RoochAddress;
use rooch_types::chain_id::{BuiltinChainID, RoochChainID};
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::GenesisError;
use rooch_types::multichain_id::RoochMultiChainID;
use serde_json::json;
use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

pub mod server;
pub mod service;

/// This exit code means is that the server failed to start and required human intervention.
static R_EXIT_CODE_NEED_HELP: i32 = 120;

pub struct ServerHandle {
    handle: jsonrpsee::server::ServerHandle,
    timers: Vec<Timer>,
    _store_config: StoreConfig,
}

impl ServerHandle {
    fn stop(self) -> Result<()> {
        self.handle.stop()?;
        for timer in self.timers {
            timer.stop();
        }
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

    pub async fn start(&mut self, opt: &RoochOpt) -> Result<()> {
        self.handle = Some(start_server(opt).await?);
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
pub async fn start_server(opt: &RoochOpt) -> Result<ServerHandle> {
    match run_start_server(opt).await {
        Ok(server_handle) => Ok(server_handle),
        Err(e) => match e.downcast::<GenesisError>() {
            Ok(e) => {
                log::error!("{:?}, please clean your data dir. `rooch server clean` ", e);
                std::process::exit(R_EXIT_CODE_NEED_HELP);
            }
            Err(e) => match e.downcast::<RawStoreError>() {
                Ok(e) => {
                    log::error!("{:?}, please clean your data dir. `rooch server clean` ", e);
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
pub async fn run_start_server(opt: &RoochOpt) -> Result<ServerHandle> {
    // We may call `start_server` multiple times in testing scenarios
    // tracing_subscriber can only be inited once.
    let _ = tracing_subscriber::fmt::try_init();

    let config = opt.port.map_or(ServerConfig::default(), |port| {
        ServerConfig::new_with_port(port)
    });
    let chain_id_opt = opt.chain_id.clone().unwrap_or_default();

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let actor_system = ActorSystem::global_system();

    //Init store
    let base_config = BaseConfig::load_with_opt(opt)?;
    let mut store_config = StoreConfig::default();
    store_config.merge_with_opt_then_init(opt, Arc::new(base_config))?;
    let (moveos_store, rooch_store) = init_storage(&store_config)?;

    // Init executor
    let is_genesis = moveos_store.statedb.is_genesis();
    let executor = ExecutorActor::new(
        chain_id_opt.genesis_ctx(),
        moveos_store,
        rooch_store.clone(),
    )?
    .into_actor(Some("Executor"), &actor_system)
    .await?;
    let executor_proxy = ExecutorProxy::new(executor.into());

    // Init sequencer
    //TODO load from config
    let (_, kp, _, _) = generate_new_key_pair::<RoochAddress, RoochKeyPair>(
        RoochMultiChainID::as_multichain(&RoochChainID::Builtin(BuiltinChainID::Dev)),
        None,
        None,
    )?;
    let sequencer = SequencerActor::new(kp, rooch_store, is_genesis)?
        .into_actor(Some("Sequencer"), &actor_system)
        .await?;
    let sequencer_proxy = SequencerProxy::new(sequencer.into());

    // Init proposer
    let (_, kp, _, _) = generate_new_key_pair::<RoochAddress, RoochKeyPair>(
        RoochMultiChainID::as_multichain(&RoochChainID::Builtin(BuiltinChainID::Dev)),
        None,
        None,
    )?;
    let proposer = ProposerActor::new(kp)
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

    let rpc_service = RpcService::new(
        chain_id_opt.chain_id().id(),
        executor_proxy,
        sequencer_proxy,
        proposer_proxy,
    );
    let aggregate_service = AggregateService::new(rpc_service.clone());

    if let Some(eth_rpc_url) = &opt.eth_rpc_url {
        //TODO load from config
        let (_, kp, _, _) = generate_new_key_pair::<RoochAddress, RoochKeyPair>(
            RoochMultiChainID::as_multichain(&RoochChainID::Builtin(BuiltinChainID::Dev)),
            None,
            None,
        )?;
        let relayer = RelayerActor::new(kp, eth_rpc_url, rpc_service.clone())
            .await?
            .into_actor(Some("Relayer"), &actor_system)
            .await?;
        let relay_tick_in_seconds: u64 = 5;
        let relayer_timer = Timer::start(
            relayer,
            Duration::from_secs(relay_tick_in_seconds),
            RelayTick {},
        );
        timers.push(relayer_timer);
    }

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
        .allow_headers([hyper::header::CONTENT_TYPE]);

    // TODO: tracing
    let middleware = tower::ServiceBuilder::new().layer(cors);

    // Build server
    let server = ServerBuilder::default()
        .set_middleware(middleware)
        .build(&addr)
        .await?;

    let mut rpc_module_builder = RpcModuleBuilder::new();
    rpc_module_builder.register_module(RoochServer::new(
        rpc_service.clone(),
        aggregate_service.clone(),
    ))?;
    rpc_module_builder.register_module(WalletServer::new(rpc_service.clone()))?;

    rpc_module_builder
        .register_module(EthServer::new(chain_id_opt.chain_id(), rpc_service.clone()))?;

    // let rpc_api = build_rpc_api(rpc_api);
    let methods_names = rpc_module_builder.module.method_names().collect::<Vec<_>>();
    let handle = server.start(rpc_module_builder.module)?;

    info!("JSON-RPC HTTP Server start listening {:?}", addr);
    info!("Available JSON-RPC methods : {:?}", methods_names);

    Ok(ServerHandle {
        handle,
        timers,
        _store_config: store_config,
    })
}

fn _build_rpc_api<M: Send + Sync + 'static>(mut rpc_module: RpcModule<M>) -> RpcModule<M> {
    let mut available_methods = rpc_module.method_names().collect::<Vec<_>>();
    available_methods.sort();

    rpc_module
        .register_method("rpc_methods", move |_, _| {
            Ok(json!({
                "methods": available_methods,
            }))
        })
        .expect("infallible all other methods have their own address space");

    rpc_module
}

fn init_storage(store_config: &StoreConfig) -> Result<(MoveOSStore, RoochStore)> {
    let (rooch_db_path, moveos_db_path) = (
        store_config.get_rooch_store_dir(),
        store_config.get_moveos_store_dir(),
    );

    //Init store
    let moveosdb = MoveOSDB::new(StoreInstance::new_db_instance(RocksDB::new(
        moveos_db_path,
        moveos_store::StoreMeta::get_column_family_names().to_vec(),
        store_config.rocksdb_config(),
        None,
    )?))?;
    let lastest_state_root = moveosdb
        .config_store
        .get_startup_info()?
        .map(|info| info.state_root_hash);

    if let Some(latest_state_root) = lastest_state_root {
        info!("Load latest state root {:?}", latest_state_root);
    }
    let moveos_store = MoveOSStore::new_with_root(moveosdb, lastest_state_root)?;

    let rooch_store = RoochStore::new(StoreInstance::new_db_instance(RocksDB::new(
        rooch_db_path,
        rooch_store::StoreMeta::get_column_family_names().to_vec(),
        store_config.rocksdb_config(),
        None,
    )?))?;
    Ok((moveos_store, rooch_store))
}
