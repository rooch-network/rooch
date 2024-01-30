// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use celestia_rpc::{BlobClient, Client};
use celestia_types::blob::SubmitOptions;
use celestia_types::nmt::Namespace;
use celestia_types::{Blob, Commitment};

use crate::segment::{Segment, SegmentID};

pub struct Backend {
    namespace: Namespace,
    client: Client,
}

pub struct SubmitBackendResult {
    pub segment_id: SegmentID,
    pub namespace: Namespace,
    pub height: u64,
    pub commitment: Commitment,
}

impl Backend {
    pub async fn new(namespace: Namespace, conn_str: &str, auth_token: &str) -> Self {
        let celestia_client = Client::new(conn_str, Option::from(auth_token))
            .await
            .unwrap();
        Self {
            namespace,
            client: celestia_client,
        }
    }

    // TODO return segment id, height, commitment
    pub async fn submit(&self, segment: Box<dyn Segment + Send>) -> Result<SubmitBackendResult> {
        let data = segment.to_bytes();
        let blob = Blob::new(self.namespace, data).unwrap();
        let segment_id = segment.get_id();

        // TODO tx manager
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
                    "failed to submit segment to celestia node, chunk: {}, segment: {}, commitment: {:?}, error:{:?}",
                    segment_id.chunk_id,
                    segment_id.segment_number,
                    blob.commitment,
                    e,
                );
                Err(e.into())
            }
        }
    }
}
