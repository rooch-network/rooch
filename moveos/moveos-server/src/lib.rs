// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod actor;

pub mod config;

pub mod error;

pub mod helper;

pub mod pb;

pub mod proxy;

pub mod response;

pub mod service;

// use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
// use jsonrpsee::rpc_params;
use jsonrpsee::server::ServerBuilder;
pub use pb::*;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use coerce::actor::{system::ActorSystem, IntoActor};
use config::Config;
use moveos::moveos::MoveOS;
use moveos_statedb::StateDB;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::Path};
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tonic::transport::Server;
use tonic::Request;
use tracing::info;

use crate::os_service_client::OsServiceClient;
use crate::{
    actor::executor::ServerActor,
    os_service_server::OsServiceServer,
    proxy::ServerProxy,
    service::{OsSvc, RoochServer, RpcServiceServer},
    HelloRequest,
};

#[async_trait]
pub trait Execute {
    type Res;
    async fn execute(&self) -> Result<Self::Res>;
}

// Start json-rpc server
pub async fn start_server() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = load_config().await?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    let actor_system = ActorSystem::global_system();
    let moveos = MoveOS::new(StateDB::new_with_memory_store())?;
    let actor = ServerActor::new(moveos)
        .into_actor(Some("Server"), &actor_system)
        .await?;
    let manager = ServerProxy::new(actor.into());
    let rpc_service = RoochServer::new(manager);
    let server = ServerBuilder::default().build(&addr).await?;

    let handle = server.start(rpc_service.into_rpc())?;

    info!("starting listening {:?}", addr);

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
    Publish(PublishPackage),
    ExecuteFunction(ExecuteFunction),
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
            Publish(publish) => {
                let _resp = publish.execute().await;
                Ok(())
            }
            ExecuteFunction(ef) => {
                let _resp = ef.execute().await;
                Ok(())
            }
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
        // let url = load_config().await?.server.url(false);

        // let client = http_client(&url)?;

        // let param = rpc_params![self.name.clone()];
        // println!("{:?}", param);

        // let response: Result<String> = client
        //     .request("echo", rpc_params![self.name.clone()])
        //     .await
        //     .map_err(|e| e.into());

        let url = load_config().await?.server.url(false);

        let mut client = OsServiceClient::connect(url).await?;
        let request = Request::new(HelloRequest {
            name: self.name.clone(),
        });

        let response = client.echo(request).await?.into_inner();
        println!("{:?}", response);

        Ok(response.message)
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
        start_grpc_server().await
    }
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct PublishPackage {
    // TODO Refactor fields to build module
    #[clap(short, long, default_value = ".")]
    pub module_path: String,
}

impl PublishPackage {
    // TODO compile module and serialize to bcs bytes
    pub fn compile_module_and_serailize(&self) -> Vec<u8> {
        self.module_path.as_bytes().to_vec()
    }
}

#[async_trait]
impl Execute for PublishPackage {
    type Res = String;
    async fn execute(&self) -> Result<Self::Res> {
        // let url = load_config().await?.server.url(false);

        // let client = http_client(&url)?;

        // // TODO compile the package and build to bcs bytes format
        // let response = client
        //     .request("publish", rpc_params!(self.module_path.as_bytes()))
        //     .await?;

        let url = load_config().await?.server.url(false);

        let mut client = OsServiceClient::connect(url).await?;

        // TODO compile the package and build to bcs bytes format
        let module_bytes = self.compile_module_and_serailize();
        let request = Request::new(PublishPackageRequest {
            module: module_bytes,
        });

        let response = client.publish(request).await?.into_inner();
        println!("{:?}", response);

        Ok(response.resp)
    }
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct ExecuteFunction {
    // TODO Refactor fields to move call
    #[clap(short, long)]
    pub move_call: String,
}

impl ExecuteFunction {
    // TODO serialize the move call to bcs bytes format
    fn serialize_function(&self) -> Vec<u8> {
        self.move_call.clone().into_bytes()
    }
}

#[async_trait]
impl Execute for ExecuteFunction {
    type Res = String;
    async fn execute(&self) -> Result<Self::Res> {
        // let url = load_config().await?.server.url(false);

        // let client = http_client(&url)?;

        let url = load_config().await?.server.url(false);

        let mut client = OsServiceClient::connect(url).await?;

        // TODO move call bcs bytes format
        let function_bytes = self.serialize_function();

        let request = Request::new(ExecutionFunctionRequest {
            functions: function_bytes,
        });

        let response = client.execute_function(request).await?.into_inner();
        println!("{:?}", response);

        Ok(response.resp)
    }
}

// Start grpc server
pub async fn start_grpc_server() -> Result<()> {
    let config = load_config().await?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    let actor_system = ActorSystem::global_system();
    let moveos = MoveOS::new(StateDB::new_with_memory_store())?;
    let actor = ServerActor::new(moveos)
        .into_actor(Some("Server"), &actor_system)
        .await?;
    let manager = ServerProxy::new(actor.into());
    let svc = OsSvc::new(manager);
    let svc = OsServiceServer::new(svc);

    println!("Listening on {addr:?}");
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

// Load config file from env or default path or default value
async fn load_config() -> Result<Config> {
    let filename = std::env::var("ROOCH_CONFIG")
        .unwrap_or_else(|_| Path::new("./rooch.yml").to_str().unwrap().to_string());
    Config::load(filename).map_err(|e| e.into())
}
