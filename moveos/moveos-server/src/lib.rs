// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod actor;
pub mod helper;
pub mod proxy;
pub mod response;
pub mod service;

use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::server::ServerBuilder;

use crate::{
    actor::executor::ServerActor,
    proxy::ServerProxy,
    service::{RoochServer, RpcServiceClient, RpcServiceServer},
};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use coerce::actor::{system::ActorSystem, IntoActor};
use moveos::moveos::MoveOS;
use moveos_common::config::load_config;
use moveos_statedb::StateDB;
use serde::{Deserialize, Serialize};
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
