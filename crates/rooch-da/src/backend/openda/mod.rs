// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

mod avail;
mod operator;

use crate::backend::openda::operator::{new_operator, Operator, OperatorConfig};
use crate::backend::DABackend;
use async_trait::async_trait;
use rooch_config::da_config::DABackendOpenDAConfig;
use rooch_types::da::batch::DABatch;
use rooch_types::da::chunk::{Chunk, ChunkV0};

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
        let scheme = operator_config.scheme.clone();
        let operator = new_operator(scheme, map_config, None).await?;

        Ok(Self {
            operator_config,
            operator,
        })
    }

    pub async fn pub_batch(&self, batch: DABatch) -> anyhow::Result<()> {
        let chunk: ChunkV0 = batch.into();

        let scheme = self.operator_config.scheme.clone();
        let prefix = self.operator_config.prefix.clone();
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
