// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::adapter::{AdapterSubmitStat, OpenDAAdapter};
use anyhow::anyhow;
use async_trait::async_trait;
use celestia_rpc::{BlobClient, Client};
use celestia_types::blob::SubmitOptions;
use celestia_types::nmt::Namespace;
use celestia_types::Blob;
use rooch_types::da::segment::SegmentID;
use std::fmt::Display;
use std::time::Duration;
use tokio::time::sleep;

// small blob size for transaction to get included in a block quickly
pub(crate) const DEFAULT_CELESTIA_MAX_SEGMENT_SIZE: u64 = 256 * 1024;
// another mechanism guarantees eventual consistency, ok to retry once
pub(crate) const DEFAULT_CELESTIA_MAX_RETRIES: usize = 1;
const BACK_OFF_MIN_DELAY: Duration = Duration::from_millis(3000);
const MAX_BACKOFF_DELAY: Duration = Duration::from_secs(30);

pub(crate) struct CelestiaAdapter {
    stats: AdapterSubmitStat,
    namespace: Namespace,
    client: Client,
    max_retries: usize,
}

impl CelestiaAdapter {
    pub async fn new(
        namespace: Namespace,
        endpoint: &str,
        auth_token: Option<&str>,
        max_retries: usize,
        stats: AdapterSubmitStat,
    ) -> anyhow::Result<Self> {
        let celestia_client = Client::new(endpoint, auth_token).await?;
        Ok(CelestiaAdapter {
            stats,
            namespace,
            client: celestia_client,
            max_retries,
        })
    }

    async fn submit(&self, segment_id: SegmentID, segment_bytes: &[u8]) -> anyhow::Result<()> {
        let blob = Blob::new(self.namespace, segment_bytes.to_vec())?;
        let max_attempts = self.max_retries + 1; // max_attempts = max_retries + first attempt
        let mut attempts = 0;
        let mut retry_delay = BACK_OFF_MIN_DELAY;

        loop {
            attempts += 1;
            match self
                .client
                .blob_submit(&[blob.clone()], SubmitOptions::default())
                .await
            {
                Ok(height) => {
                    tracing::info!(
                        "submitted segment to Celestia node, segment_id: {:?}, commitment: {:?}, height: {:?}",
                        segment_id,
                        blob.commitment,
                        height,
                    );
                    return Ok(());
                }
                Err(e) => {
                    if attempts < max_attempts {
                        tracing::warn!(
                            "Failed to submit segment: {:?} to Celestia: {:?}, attempts: {}, retrying after {}ms",
                            segment_id,
                            e,
                            attempts,
                            retry_delay.as_millis(),
                        );
                        sleep(retry_delay).await;
                        retry_delay = std::cmp::min(retry_delay * 2, MAX_BACKOFF_DELAY);
                    } else {
                        return Err(anyhow!(
                            "Failed to submit segment: {:?} to Celestia: {:?} after {} attempts",
                            segment_id,
                            e,
                            attempts,
                        ));
                    }
                }
            }
        }
    }
}

#[async_trait]
impl OpenDAAdapter for CelestiaAdapter {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: &[u8],
        is_last_segment: bool,
    ) -> anyhow::Result<()> {
        match self.submit(segment_id, segment_bytes).await {
            Ok(_) => {
                self.stats
                    .add_done_segment(segment_id, is_last_segment)
                    .await;
                Ok(())
            }
            Err(error) => Err(error),
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
