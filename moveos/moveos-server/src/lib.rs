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
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::RpcModule;

use crate::api::account::AccountServer;
use crate::api::RoochRpcModule;

use crate::{
    actor::executor::ServerActor,
    proxy::ServerProxy,
    service::{RoochServer, RpcServiceClient},
};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use coerce::actor::{system::ActorSystem, IntoActor};
use moveos::moveos::MoveOS;
use moveos_common::config::load_config;
use moveos_statedb::StateDB;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

#[async_trait]
pub trait Execute {
    type Res;
    async fn execute(&self) -> Result<Self::Res>;
}

/// For grpc
#[derive(Debug, Parser)]
pub struct OsServer {
    #[clap(subcommand)]
    pub command: Command,
}

impl OsServer {
    pub async fn execute(&self) -> Result<()> {
        self.command.execute().await
    }
}

#[derive(Debug, Parser)]
pub enum Command {
    Say(SayOptions),
    Start(Start),
}

#[async_trait]
impl Execute for Command {
    type Res = ();
    async fn execute(&self) -> Result<()> {
        use Command::*;
        match self {
            Say(say) => {
                let _resp = say.execute().await;
                Ok(())
            }
            Start(start) => start.execute().await,
        }
    }
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct SayOptions {
    #[clap(short, long, default_value = "hello")]
    name: String,
}

#[async_trait]
impl Execute for SayOptions {
    type Res = String;

    // Test server liveness
    async fn execute(&self) -> Result<Self::Res> {
        let url = load_config()?.server.url(false);
        println!("client url: {:?}", url);

        let client = http_client(url)?;
        let resp = client
            .echo("Hello rooch".to_string())
            .await
            .unwrap_or_else(|e| panic!("{:?}", e));
        println!("{:?}", resp);

        // let mut client = OsServiceClient::connect(url).await?;
        // let request = Request::new(HelloRequest {
        //     name: self.name.clone(),
        // });

        // let response = client.echo(request).await?.into_inner();
        // println!("{:?}", response);

        Ok("ok".to_string())
    }
}

pub fn http_client(url: impl AsRef<str>) -> Result<HttpClient> {
    let client = HttpClientBuilder::default().build(url)?;
    Ok(client)
}

// pub async fn http_request<R, Params>(url: impl AsRef<str>, method: impl AsRef<str>, params: Params) -> Result<R, Error>
// where
// 		R: DeserializeOwned,
// 		Params: ToRpcParams + Send,
// {
//     let client = http_client(&url)?;

//     let response: Result<R, Error> = client.request(method.as_ref(), params)
//         .await;
//         // .map_err(|e| e.into());

//     response
// }

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Start {}

#[async_trait]
impl Execute for Start {
    type Res = ();
    async fn execute(&self) -> Result<Self::Res> {
        start_server().await
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
pub async fn start_server() -> Result<()> {
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

    let mut sig_int = signal(SignalKind::interrupt()).unwrap();
    let mut sig_term = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sig_int.recv() => info!("receive SIGINT"),
        _ = sig_term.recv() => info!("receive SIGTERM"),
        _ = ctrl_c() => info!("receive Ctrl C"),
    }

    handle.stop().unwrap();

    info!("Shutdown Sever");

    Ok(())
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
