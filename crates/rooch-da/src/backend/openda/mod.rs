// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

mod avail;
mod celestia;
mod fs;
mod operator;

use crate::backend::openda::avail::AvailClient;
use crate::backend::openda::celestia::{CelestiaClient, WrappedNamespace};
use crate::backend::openda::operator::{Operator, OperatorConfig};
use crate::backend::DABackend;
use async_trait::async_trait;
use opendal::layers::{LoggingLayer, RetryLayer};
use opendal::Scheme;
use rooch_config::da_config::{DABackendOpenDAConfig, OpenDAScheme};
use rooch_types::da::batch::DABatch;
use rooch_types::da::chunk::{Chunk, ChunkV0};
use std::collections::HashMap;

#[async_trait]
impl DABackend for OpenDABackend {
    async fn submit_batch(&self, batch: DABatch) -> anyhow::Result<()> {
        self.pub_batch(batch).await
    }
}

pub struct OpenDABackend {
    operator_config: OperatorConfig,
    operator: Box<dyn Operator>,
}

impl OpenDABackend {
    pub async fn new(
        cfg: &DABackendOpenDAConfig,
        genesis_namespace: String,
    ) -> anyhow::Result<OpenDABackend> {
        let (operator_config, map_config) =
            OperatorConfig::from_backend_config(cfg.clone(), genesis_namespace)?;
        let operator = new_operator(operator_config.clone(), map_config).await?;

        Ok(Self {
            operator_config,
            operator,
        })
    }

    pub async fn pub_batch(&self, batch: DABatch) -> anyhow::Result<()> {
        let chunk: ChunkV0 = batch.into();

        let scheme = self.operator_config.scheme.clone();
        let prefix = self.operator_config.namespace.clone();
        let max_segment_size = self.operator_config.max_segment_size;

        let segments = chunk.to_segments(max_segment_size);
        for segment in segments {
            let bytes = segment.to_bytes();

            match self
                .operator
                .submit_segment(segment.get_id(), bytes, Some(prefix.clone()))
                .await
            {
                Ok(_) => {
                    tracing::info!(
                        "submitted segment to open-da scheme: {:?}, segment_id: {:?}",
                        scheme,
                        segment.get_id(),
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "failed to submit segment to open-da scheme: {:?}, segment_id: {:?}, error:{:?}",
                        scheme,
                        segment.get_id(),
                        e,
                    );
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

async fn new_operator(
    operator_config: OperatorConfig,
    config: HashMap<String, String>,
) -> anyhow::Result<Box<dyn Operator>> {
    let max_retries = operator_config.max_retries;
    let scheme = operator_config.scheme.clone();

    let operator: Box<dyn Operator> = match scheme {
        OpenDAScheme::Avail => Box::new(AvailClient::new(&config["endpoint"], max_retries)?),
        OpenDAScheme::Celestia => {
            let namespace = WrappedNamespace::from_string(&operator_config.namespace.clone())?;
            Box::new(
                CelestiaClient::new(
                    namespace.into_inner(),
                    &config["endpoint"],
                    config.get("auth_token").map(|s| s.as_str()),
                )
                .await?,
            )
        }
        _ => {
            let mut op = opendal::Operator::via_iter(Scheme::from(scheme), config)?;
            op = op
                .layer(RetryLayer::new().with_max_times(max_retries))
                .layer(LoggingLayer::default());
            op.check().await?;
            Box::new(op)
        }
    };
    Ok(operator)
}
