// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::system::ActorSystem;
use coerce::actor::{Actor, IntoActor};
use futures::stream::FuturesUnordered;
use futures::StreamExt;

use rooch_config::da_config::{DAConfig, InternalDAServerConfigType};

use crate::messages::{Batch, PutBatchInternalDAMessage};
use crate::server::celestia::actor::server::DAServerCelestiaActor;
use crate::server::celestia::proxy::DAServerCelestiaProxy;
use crate::server::openda::actor::server::DAServerOpenDAActor;
use crate::server::openda::proxy::DAServerOpenDAProxy;
use crate::server::serverproxy::DAServerProxy;

// TODO tx buffer for building batch
pub struct DAActor {
    internal_servers: InternalServers,
}

struct InternalServers {
    servers: Arc<RwLock<Vec<Arc<dyn DAServerProxy + Send + Sync>>>>,
    submit_threshold: usize,
}

impl Actor for DAActor {}

impl DAActor {
    pub async fn new(da_config: DAConfig, actor_system: &ActorSystem) -> Result<Self> {
        // internal servers

        let mut servers: Vec<Arc<dyn DAServerProxy + Send + Sync>> = Vec::new();
        let mut submit_threshold = 1;
        let mut success_count = 0;

        // server config has higher priority than submit threshold
        if let Some(internal_da_server_config) = &da_config.internal_da_server {
            let mut server_config = internal_da_server_config.clone();
            submit_threshold = server_config.calculate_submit_threshold();

            for server_config_type in &server_config.servers {
                if let InternalDAServerConfigType::Celestia(celestia_config) = server_config_type {
                    let da_server = DAServerCelestiaActor::new(celestia_config)
                        .await
                        .into_actor(Some("DAServerCelestia"), actor_system)
                        .await?;
                    servers.push(Arc::new(DAServerCelestiaProxy::new(
                        da_server.clone().into(),
                    )));
                    success_count += 1;
                }
                if let InternalDAServerConfigType::OpenDa(openda_config) = server_config_type {
                    let da_server = DAServerOpenDAActor::new(openda_config)
                        .await?
                        .into_actor(
                            Some(format!("DAServerOpenDA-{}", openda_config.scheme)),
                            actor_system,
                        )
                        .await?;
                    servers.push(Arc::new(DAServerOpenDAProxy::new(da_server.clone().into())));
                    success_count += 1;
                }
            }
        } else {
            servers.push(Arc::new(crate::server::serverproxy::DAServerNopProxy {}));
        }

        if success_count < submit_threshold {
            return Err(anyhow!(
                "failed to start da: not enough servers for future submissions. exp>= {} act: {}",
                submit_threshold,
                success_count
            ));
        }

        Ok(Self {
            internal_servers: InternalServers {
                servers: Arc::new(RwLock::new(servers)),
                submit_threshold,
            },
        })
    }

    pub async fn submit_batch(&self, batch: Batch) -> Result<()> {
        // TODO calc checksum
        // TODO richer policy for multi servers
        // TODO verify checksum
        // TODO retry policy & log

        let servers = self.internal_servers.servers.read().unwrap().to_vec();
        let submit_threshold = self.internal_servers.submit_threshold;

        let mut futures_unordered = FuturesUnordered::new();
        for server in servers {
            let server = Arc::clone(&server);
            let batch = batch.clone();
            futures_unordered.push(async move {
                server
                    .public_batch(PutBatchInternalDAMessage {
                        batch: batch.clone(),
                    })
                    .await
            });
        }

        let mut success_count = 0;
        while let Some(result) = futures_unordered.next().await {
            match result {
                Ok(_) => {
                    success_count += 1;
                    if success_count >= submit_threshold {
                        return Ok(());
                    }
                }
                Err(e) => {
                    log::warn!("{:?}, fail to submit batch to da server.", e); // TODO add da server name
                }
            }
        }

        if success_count < submit_threshold {
            Err(anyhow::Error::msg(format!(
                "not enough successful submissions. exp>= {} act: {}",
                submit_threshold, success_count
            )))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl Handler<Batch> for DAActor {
    async fn handle(&mut self, msg: Batch, _ctx: &mut ActorContext) -> Result<()> {
        self.submit_batch(msg).await
    }
}
