// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::adapter::{OpenDAAdapter, OpenDAAdapterConfig};
use crate::backend::openda::derive_identifier;
use crate::backend::DABackend;
use async_trait::async_trait;
use rooch_config::da_config::DABackendOpenDAConfig;
use rooch_types::da::batch::DABatch;
use rooch_types::da::chunk::{Chunk, ChunkV0};
use std::sync::Arc;

/// manage OpenDA backends while integrating specific adapter logic
pub struct OpenDABackendManager {
    identifier: String,
    adapter_config: OpenDAAdapterConfig,
    adapter: Box<dyn OpenDAAdapter>,
}

impl OpenDABackendManager {
    pub async fn new(
        open_da_config: &DABackendOpenDAConfig,
    ) -> anyhow::Result<OpenDABackendManager> {
        let adapter_config = OpenDAAdapterConfig::derive_from_open_da_config(open_da_config)?;
        let adapter = adapter_config.build().await?;

        Ok(Self {
            identifier: derive_identifier(open_da_config.scheme.clone()),
            adapter_config,
            adapter,
        })
    }
}

#[async_trait]
impl DABackend for OpenDABackendManager {
    async fn submit_batch(&self, batch: Arc<DABatch>) -> anyhow::Result<()> {
        let chunk: ChunkV0 = (*batch).clone().into();

        let max_segment_size = self.adapter_config.max_segment_size;

        let segments = chunk.to_segments(max_segment_size);
        for segment in segments {
            let bytes = segment.to_bytes();

            match self.adapter.submit_segment(segment.get_id(), &bytes).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        "failed to submit segment to {:?}, segment_id: {:?}, error:{:?}",
                        self.identifier,
                        segment.get_id(),
                        e,
                    );
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn get_identifier(&self) -> String {
        self.identifier.clone()
    }
}
