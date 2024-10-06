// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::OpenDABackend;
use crate::backend::DABackend;
use async_trait::async_trait;
use celestia_rpc::{BlobClient, Client};
use celestia_types::blob::SubmitOptions;
use celestia_types::nmt::Namespace;
use celestia_types::{Blob, Commitment};
use rooch_config::da_config::DABackendCelestiaConfig;
use rooch_types::da::batch::DABatch;
use rooch_types::da::chunk::{Chunk, ChunkV0};
use rooch_types::da::segment::{Segment, SegmentID};
use std::cmp::max;

pub struct CelestiaBackend {
    max_segment_size: usize,
    client: CelestiaClient,
}

#[async_trait]
impl DABackend for CelestiaBackend {
    async fn submit_batch(&self, batch: DABatch) -> anyhow::Result<()> {
        let chunk: ChunkV0 = batch.into();
        let segments = chunk.to_segments(self.max_segment_size);
        for segment in segments {
            let result = self.client.submit(segment).await?;
            log::info!(
                "submitted segment to celestia node, segment_id: {:?}, namespace: {:?}, commitment: {:?}, height: {}",
                result.segment_id,
                result.namespace,
                result.commitment,
                result.height,
            );
        }

        Ok(())
    }
}

impl CelestiaBackend {
    pub async fn new(cfg: &DABackendCelestiaConfig) -> anyhow::Result<Self> {
        let namespace_str = cfg.namespace.as_ref().unwrap().clone();
        let namespace: Namespace = serde_yaml::from_str(&namespace_str)?;
        let conn_str = cfg.conn.as_ref().unwrap().clone();
        let token = cfg.auth_token.as_ref().unwrap().clone();

        let max_segment_size = cfg.max_segment_size.unwrap() as usize;
        let client = CelestiaClient::new(namespace, &conn_str, &token).await?;

        Ok(CelestiaBackend {
            max_segment_size,
            client,
        })
    }
}

struct CelestiaClient {
    namespace: Namespace,
    client: Client,
}

pub struct SubmitBackendResult {
    pub segment_id: SegmentID,
    pub namespace: Namespace,
    pub height: u64,
    pub commitment: Commitment,
}

impl CelestiaClient {
    pub async fn new(
        namespace: Namespace,
        conn_str: &str,
        auth_token: &str,
    ) -> anyhow::Result<Self> {
        let celestia_client = Client::new(conn_str, Option::from(auth_token)).await?;
        Ok(CelestiaClient {
            namespace,
            client: celestia_client,
        })
    }

    pub async fn submit(
        &self,
        segment: Box<dyn Segment + Send>,
    ) -> anyhow::Result<SubmitBackendResult> {
        let data = segment.to_bytes();
        let blob = Blob::new(self.namespace, data).unwrap();
        let segment_id = segment.get_id();
        // TODO backoff retry
        match self
            .client
            .blob_submit(&[blob.clone()], SubmitOptions::default())
            .await
        {
            Ok(height) => Ok(SubmitBackendResult {
                segment_id,
                namespace: self.namespace,
                height,
                commitment: blob.commitment,
            }),
            Err(e) => {
                log::warn!(
                    "failed to submit segment to celestia node, segment_id: {:?}, commitment: {:?}, error:{:?}",
                    segment_id,
                    blob.commitment,
                    e,
                );
                Err(e.into())
            }
        }
    }
}
