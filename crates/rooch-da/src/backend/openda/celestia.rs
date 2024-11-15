// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::operator::Operator;
use async_trait::async_trait;
use celestia_rpc::{BlobClient, Client};
use celestia_types::blob::SubmitOptions;
use celestia_types::nmt::Namespace;
use celestia_types::Blob;
use rooch_types::da::segment::SegmentID;
use std::fmt::Display;

// small blob size for transaction to get included in a block quickly
pub(crate) const DEFAULT_CELESTIA_MAX_SEGMENT_SIZE: u64 = 256 * 1024;

pub(crate) struct CelestiaClient {
    namespace: Namespace,
    client: Client,
}

impl CelestiaClient {
    pub async fn new(
        namespace: Namespace,
        endpoint: &str,
        auth_token: Option<&str>,
    ) -> anyhow::Result<Self> {
        let celestia_client = Client::new(endpoint, auth_token).await?;
        Ok(CelestiaClient {
            namespace,
            client: celestia_client,
        })
    }
}

#[async_trait]
impl Operator for CelestiaClient {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: Vec<u8>,
        _prefix: Option<String>,
    ) -> anyhow::Result<()> {
        let blob = Blob::new(self.namespace, segment_bytes)?;
        // TODO backoff retry
        match self
            .client
            .blob_submit(&[blob.clone()], SubmitOptions::default())
            .await
        {
            Ok(height) => {
                tracing::info!(
                    "submitted segment to celestia node, segment_id: {:?}, commitment: {:?}, height: {:?}",
                    segment_id,
                    blob.commitment,
                    height,
                );
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
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

pub(crate) struct WrappedNamespace(Namespace);

impl WrappedNamespace {
    pub(crate) fn from_string(s: &str) -> anyhow::Result<Self> {
        let decoded_bytes = hex::decode(s)?;
        Ok(WrappedNamespace(Namespace::from_raw(&decoded_bytes)?))
    }

    pub(crate) fn into_inner(self) -> Namespace {
        self.0
    }
}

impl Display for WrappedNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode((self.0).0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapped_namespace() {
        let nid = Namespace::new_v0(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // prefix
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
        ])
        .unwrap();

        let wrapped_namespace = WrappedNamespace(nid);
        let nid_str = wrapped_namespace.to_string();

        assert_eq!(
            WrappedNamespace::from_string(&nid_str)
                .unwrap()
                .into_inner(),
            nid
        );
    }
}
