// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::PutDABatchMessage;
use crate::backend::celestia::CelestiaBackend;
use crate::backend::openda::OpenDABackend;
use crate::backend::DABackend;
use anyhow::anyhow;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::system::ActorSystem;
use coerce::actor::{Actor, IntoActor};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use rooch_config::da_config::{DABackendConfigType, DAConfig};
use rooch_store::da_store::{DAMetaDBStore, DAMetaStore};
use rooch_store::RoochStore;
use rooch_types::crypto::RoochKeyPair;
use std::sync::{Arc, RwLock};

pub struct DAServerActor {
    sequencer_key: RoochKeyPair,
    da_meta_store: *DAMetaDBStore,
    backends: DABackends,
}

struct DABackends {
    backends: Arc<RwLock<Vec<Arc<dyn DABackend + Send + Sync>>>>,
    submit_threshold: usize,
}

impl Actor for DAServerActor {}

impl DAServerActor {
    pub async fn new(
        da_config: DAConfig,
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
    ) -> anyhow::Result<Self> {
        let mut backends: Vec<Arc<dyn DABackend + Send + Sync>> = Vec::new();
        let mut submit_threshold = 1;
        let mut act_backends = 0;

        // backend config has higher priority than submit threshold
        if let Some(mut backend_config) = &da_config.da_backend {
            submit_threshold = backend_config.calculate_submit_threshold();

            for backend_type in &backend_config.backends {
                if let DABackendConfigType::Celestia(celestia_config) = backend_type {
                    let backend = CelestiaBackend::new(celestia_config).await?;
                    backends.push(Arc::new(backend));
                    act_backends += 1;
                }
                if let DABackendConfigType::OpenDa(openda_config) = backend_type {
                    let backend = OpenDABackend::new(openda_config).await?;
                    backends.push(Arc::new(backend));
                    act_backends += 1;
                }
            }
        } else {
            backends.push(Arc::new(crate::backend::DABackendNopProxy {}));
            act_backends += 1;
        }

        if act_backends < submit_threshold {
            return Err(anyhow!(
                "failed to start da: not enough backends for future submissions. exp>= {} act: {}",
                submit_threshold,
                act_backends
            ));
        }

        Ok(Self {
            sequencer_key,
            da_meta_store: rooch_store.get_da_meta_store(),
            backends: DABackends {
                backends: Arc::new(RwLock::new(backends)),
                submit_threshold,
            },
        })
    }

    pub async fn submit_batch(&self, msg: PutDABatchMessage) -> anyhow::Result<()> {


        let backends = self.backends.backends.read()?.to_vec();
        let submit_threshold = self.backends.submit_threshold;
        let batch = msg.batch;

        let mut submit_jobs = FuturesUnordered::new();
        for backend_arc in backends {
            let backend = Arc::clone(&backend_arc);
            submit_jobs.push(async move { backend.submit_batch(batch.clone()) });
        }

        let mut success_count = 0;
        while let Some(result) = submit_jobs.next().await {
            match result {
                Ok(_) => {
                    success_count += 1;
                    if success_count >= submit_threshold {
                        return Ok(());
                    }
                }
                Err(e) => {
                    log::warn!("{:?}, fail to submit batch to da server.", e);
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
impl Handler<PutDABatchMessage> for DAServerActor {
    async fn handle(
        &mut self,
        msg: PutDABatchMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<()> {
        self.submit_batch(msg).await
    }
}
