// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::adapter::{AdapterSubmitStat, OpenDAAdapter, OpenDAAdapterConfig};
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
    adapter_stats: AdapterSubmitStat,
    adapter_config: OpenDAAdapterConfig,
    adapter: Box<dyn OpenDAAdapter>,
}

impl OpenDABackendManager {
    pub async fn new(
        open_da_config: &DABackendOpenDAConfig,
    ) -> anyhow::Result<OpenDABackendManager> {
        let adapter_config = OpenDAAdapterConfig::derive_from_open_da_config(open_da_config)?;
        let adapter_stats = AdapterSubmitStat::new();

        let adapter = adapter_config.build(adapter_stats.clone()).await?;

        Ok(Self {
            identifier: derive_identifier(open_da_config.scheme.clone()),
            adapter_stats: adapter_stats.clone(),
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
        let segment_count = segments.len() as u64;
        for segment in segments {
            let bytes = segment.to_bytes();
            let segment_id = segment.get_id();
            let is_last_segment = segment_id.segment_number == segment_count - 1;
            match self
                .adapter
                .submit_segment(segment.get_id(), &bytes, is_last_segment)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        "failed to submit segment to {:?}, segment_id: {:?}, error:{:?}",
                        self.get_identifier(),
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

    fn get_adapter_stats(&self) -> AdapterSubmitStat {
        self.adapter_stats.clone()
    }
}
