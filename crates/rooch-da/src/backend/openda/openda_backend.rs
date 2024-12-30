// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::avail::AvailFusionClientConfig;
use crate::backend::openda::celestia::{CelestiaClient, WrappedNamespace};
use crate::backend::openda::opendal::BACK_OFF_MIN_DELAY;
use crate::backend::openda::operator::{new_operator, Operator, OperatorConfig};
use crate::backend::DABackend;
use async_trait::async_trait;
use opendal::layers::{LoggingLayer, RetryLayer};
use opendal::Scheme;
use rooch_config::da_config::{DABackendOpenDAConfig, OpenDAScheme};
use rooch_types::da::batch::DABatch;
use rooch_types::da::chunk::{Chunk, ChunkV0};
use std::collections::HashMap;
use std::sync::Arc;

pub struct OpenDABackend {
    operator_config: OperatorConfig,
    operator: Box<dyn Operator>,
}

impl OpenDABackend {
    pub async fn new(
        cfg: &DABackendOpenDAConfig,
        genesis_namespace: String,
    ) -> anyhow::Result<OpenDABackend> {
        let (operator_config, scheme_config) =
            OperatorConfig::from_backend_config(cfg.clone(), genesis_namespace)?;
        let operator = new_operator(operator_config.clone(), scheme_config).await?;

        Ok(Self {
            operator_config,
            operator,
        })
    }
}

#[async_trait]
impl DABackend for OpenDABackend {
    async fn submit_batch(&self, batch: Arc<DABatch>) -> anyhow::Result<()> {
        let chunk: ChunkV0 = (*batch).clone().into();

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
