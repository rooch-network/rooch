// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod actor;
pub mod api;
pub mod helper;
pub mod jsonrpc_types;
pub mod proxy;
pub mod response;
pub mod service;

use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::RpcModule;

use crate::api::account::AccountServer;
use crate::api::RoochRpcModule;

use crate::{actor::executor::ServerActor, proxy::ServerProxy, service::RoochServer};
use anyhow::Result;
use coerce::actor::{system::ActorSystem, IntoActor};
use moveos::moveos::MoveOS;
use moveos_common::config::load_config;
use moveos_statedb::StateDB;
use serde_json::json;
use std::net::SocketAddr;

use tracing::info;

pub fn http_client(url: impl AsRef<str>) -> Result<HttpClient> {
    let client = HttpClientBuilder::default().build(url)?;
    Ok(client)
}

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

    let config = load_config()?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    let actor_system = ActorSystem::global_system();
    let moveos = MoveOS::new(StateDB::new_with_memory_store())?;
    let actor = ServerActor::new(moveos)
        .into_actor(Some("Server"), &actor_system)
        .await?;
    let manager = ServerProxy::new(actor.into());
    let server = ServerBuilder::default().build(&addr).await?;

    let mut rpc_module_builder = RpcModuleBuilder::new();
    rpc_module_builder
        .register_module(RoochServer::new(manager.clone()))
        .unwrap();
    rpc_module_builder
        .register_module(AccountServer::new(manager.clone()))
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
