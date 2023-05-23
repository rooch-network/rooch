// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod api;
pub mod jsonrpc_types;
pub mod response;
pub mod service;

use crate::api::account::AccountServer;
use crate::api::RoochRpcModule;
use crate::service::RoochServer;
use anyhow::Result;
use coerce::actor::{system::ActorSystem, IntoActor};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::RpcModule;
use moveos::moveos::MoveOS;
use rooch_executor::actor::executor::ExecutorActor;
use rooch_executor::proxy::ExecutorProxy;
use rooch_sequencer::actor::sequencer::SequencerActor;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::transaction::authenticator::AccountPrivateKey;
//use moveos_common::config::load_config;
use moveos_store::state_store::StateDB;
use rooch_common::config::{rooch_config_path, PersistedConfig, RoochConfig};
use serde_json::json;
use std::net::SocketAddr;

use tracing::info;

pub fn http_client(url: impl AsRef<str>) -> Result<HttpClient> {
    let client = HttpClientBuilder::default().build(url)?;
    Ok(client)
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

    pub fn stop(&self) -> Result<()> {
        if let Some(handle) = &self.handle {
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

    let config: RoochConfig = PersistedConfig::read(rooch_config_path()?.as_path())?;
    let server = config.server.unwrap();
    let addr: SocketAddr = format!("{}:{}", server.host, server.port).parse()?;

    let actor_system = ActorSystem::global_system();

    // Init executor
    let moveos = MoveOS::new(StateDB::new_with_memory_store())?;
    let executor = ExecutorActor::new(moveos)
        .into_actor(Some("Executor"), &actor_system)
        .await?;
    let executor_proxy = ExecutorProxy::new(executor.into());

    //TODO load from config
    let sequencer_account = AccountPrivateKey::generate_for_testing();

    // Init sequencer
    let sequencer = SequencerActor::new(sequencer_account)
        .into_actor(Some("Sequencer"), &actor_system)
        .await?;
    let sequencer_proxy = SequencerProxy::new(sequencer.into());

    // Build server
    let server = ServerBuilder::default().build(&addr).await?;

    let mut rpc_module_builder = RpcModuleBuilder::new();
    rpc_module_builder
        .register_module(RoochServer::new(executor_proxy.clone(), sequencer_proxy))
        .unwrap();
    rpc_module_builder
        .register_module(AccountServer::new(executor_proxy.clone()))
        .unwrap();
    // let rpc_api = build_rpc_api(rpc_api);
    let methods_names = rpc_module_builder.module.method_names().collect::<Vec<_>>();
    let handle = server.start(rpc_module_builder.module)?;

    info!("JSON-RPC HTTP Server start listening {:?}", addr);
    info!("Available JSON-RPC methods : {:?}", methods_names);

    Ok(handle)
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
