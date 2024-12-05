// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::operator::Operator;
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
// default retry duration(seconds): 3, 9, 27, 81
// 81s > 60s(5 blocks) for:
// By default, nodes will drop a transaction if it does not get included in 5 blocks (roughly 1 minute).
// At this point, the user must resubmit their transaction if they want it to eventually be included.
const BACK_OFF_MIN_DELAY: Duration = Duration::from_millis(3000);

pub(crate) struct CelestiaClient {
    namespace: Namespace,
    client: Client,
    max_retries: usize,
}

impl CelestiaClient {
    pub async fn new(
        namespace: Namespace,
        endpoint: &str,
        auth_token: Option<&str>,
        max_retries: usize,
    ) -> anyhow::Result<Self> {
        let celestia_client = Client::new(endpoint, auth_token).await?;
        Ok(CelestiaClient {
            namespace,
            client: celestia_client,
            max_retries,
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
                        retry_delay *= 3;
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
