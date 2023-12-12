// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use celestia_rpc::{BlobClient, Client};
use celestia_types::Blob;
use celestia_types::blob::SubmitOptions;
use celestia_types::nmt::Namespace;

use crate::server::segment::Segment;

struct Backend {
    namespace: Namespace,
    client: Client,
}

impl Backend {
    async fn new(namespace: Namespace, conn_str: &str, auth_token: &str) -> Self {
        let celestia_client = Client::new(conn_str, Option::from(auth_token)).await.unwrap();
        Self { namespace, client: celestia_client }
    }
    
    async fn submit(&self, segment: Segment) -> Result<()> {
        let data = bcs::to_bytes(&segment).unwrap();
        let blob = Blob::new(self.namespace, data).unwrap();

        // TODO backoff retry
        return match self.client.blob_submit(&[blob.clone()], SubmitOptions::default()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::warn!(
                    "failed to submit segment to celestia node, chunk: {}, segment: {}, commitment: {:?}, error:{:?}",
                    segment.chunk_id,
                    segment.segment_id,
                    blob.commitment,
                    e,
                );
                Err(e.into())
            }
        };
    }
}