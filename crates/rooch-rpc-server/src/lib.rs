// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::server::eth_server::EthServer;
use crate::server::rooch_server::RoochServer;
use crate::server::wallet_server::WalletServer;
use crate::service::RpcService;
use anyhow::Result;
use coerce::actor::scheduler::timer::Timer;
use coerce::actor::{system::ActorSystem, IntoActor};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::RpcModule;
use rooch_config::rpc::server_config::ServerConfig;
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_key::key_derive::generate_new_key;
use rooch_proposer::actor::messages::ProposeBlock;
use rooch_proposer::actor::proposer::ProposerActor;
use rooch_proposer::proxy::ProposerProxy;
use rooch_rpc_api::api::RoochRpcModule;
use rooch_sequencer::actor::sequencer::SequencerActor;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_store::RoochDB;
use serde_json::json;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::info;

pub mod server;
pub mod service;

pub fn http_client(url: impl AsRef<str>) -> Result<HttpClient> {
    let client = HttpClientBuilder::default().build(url)?;
    Ok(client)
}

pub struct ServerHandle {
    handle: jsonrpsee::server::ServerHandle,
    timer: Timer,
}

impl ServerHandle {
    fn stop(self) -> Result<()> {
        self.handle.stop()?;
        self.timer.stop();
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

    pub async fn start(&mut self) -> Result<()> {
        self.handle = Some(start_server().await?);
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
pub async fn start_server() -> Result<ServerHandle> {
    tracing_subscriber::fmt::init();

    let config = ServerConfig::default();

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let rooch_db = RoochDB::new_with_memory_store();
    let actor_system = ActorSystem::global_system();

    // Init executor
    let executor = ExecutorActor::new(rooch_db.clone())?
        .into_actor(Some("Executor"), &actor_system)
        .await?;
    let executor_proxy = ExecutorProxy::new(executor.into());

    // Init sequencer
    //TODO load from config

    let (_, kp, _, _) = generate_new_key(rooch_types::crypto::BuiltinScheme::Ed25519, None, None)?;
    // let rooch_db = RoochDB::new_with_memory_store();
    let sequencer = SequencerActor::new(kp, rooch_db)
        .into_actor(Some("Sequencer"), &actor_system)
        .await?;
    let sequencer_proxy = SequencerProxy::new(sequencer.into());

    // Init proposer
    let (_, kp, _, _) = generate_new_key(rooch_types::crypto::BuiltinScheme::Ed25519, None, None)?;
    let proposer = ProposerActor::new(kp)
        .into_actor(Some("Proposer"), &actor_system)
        .await?;
    let proposer_proxy = ProposerProxy::new(proposer.clone().into());
    //TODO load from config
    let block_propose_duration_in_seconds: u64 = 5;
    //TODO stop timer
    let timer = Timer::start(
        proposer,
        Duration::from_secs(block_propose_duration_in_seconds),
        ProposeBlock {},
    );

    let rpc_service = RpcService::new(executor_proxy, sequencer_proxy, proposer_proxy);

    // Build server
    let server = ServerBuilder::default().build(&addr).await?;

    let mut rpc_module_builder = RpcModuleBuilder::new();
    rpc_module_builder
        .register_module(RoochServer::new(rpc_service.clone()))
        .unwrap();
    rpc_module_builder
        .register_module(WalletServer::new(rpc_service.clone()))
        .unwrap();

    rpc_module_builder
        .register_module(EthServer::new(rpc_service.clone()))
        .unwrap();

    // let rpc_api = build_rpc_api(rpc_api);
    let methods_names = rpc_module_builder.module.method_names().collect::<Vec<_>>();
    let handle = server.start(rpc_module_builder.module)?;

    info!("JSON-RPC HTTP Server start listening {:?}", addr);
    info!("Available JSON-RPC methods : {:?}", methods_names);

    Ok(ServerHandle { handle, timer })
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
