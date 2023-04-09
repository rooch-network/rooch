// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod actor;

pub mod config;

pub mod error;

pub mod helper;

pub mod pb;

pub mod proxy;

pub mod service;

pub use pb::*;

use anyhow::Result;
use coerce::actor::new_actor;
use config::Config;
use moveos::moveos::MoveOS;
use moveos_statedb::StateDB;
use std::{net::SocketAddr, path::Path};
use tonic::transport::Server;

use crate::{
    actor::executor::ServerActor, os_service_client::OsServiceClient,
    os_service_server::OsServiceServer, proxy::ServerProxy, service::OsSvc, HelloRequest,
};

use clap::Parser;
use tonic::Request;

#[derive(Debug, Parser)]
pub struct OsServer {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    Say(SayOptions),
    Start(Start),
}

#[derive(Debug, Parser)]
pub struct SayOptions {
    #[clap(short, long, default_value = "hello")]
    name: String,
}

#[derive(Debug, Parser)]
pub struct Start {}

// Test server liveness
pub async fn say_hello(say: SayOptions) -> Result<()> {
    let url = load_config().await?.server.url(false);

    let mut client = OsServiceClient::connect(url).await?;
    let request = Request::new(HelloRequest { name: say.name });

    let response = client.echo(request).await?.into_inner();
    println!("{:?}", response);

    Ok(())
}

// Start MoveOS Server
pub async fn start_server() -> Result<()> {
    let config = load_config().await?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    let actor = new_actor(ServerActor::default()).await?;
    let moveos = MoveOS::new(StateDB::new_with_memory_store())?;
    let manager = ServerProxy::new(moveos, actor.into());
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
