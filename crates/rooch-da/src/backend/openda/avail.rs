// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::operator::Operator;
use anyhow::anyhow;
use async_trait::async_trait;
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::{Client, StatusCode};
use rooch_types::da::segment::SegmentID;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

// small blob size for transaction to get included in a block quickly
pub(crate) const DEFAULT_AVAIL_MAX_SEGMENT_SIZE: u64 = 256 * 1024;
const SUBMIT_PATH: &str = "v2/submit";

pub(crate) struct AvailClient {
    endpoint: String,
    client: Client,
    max_retries: usize,
}

impl AvailClient {
    pub(crate) fn new(endpoint: &str, max_retries: usize) -> anyhow::Result<Self> {
        let client = Client::new();

        Ok(AvailClient {
            endpoint: endpoint.to_string(),
            client,
            max_retries,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailSubmitResponse {
    block_number: u32,
    block_hash: String,
    hash: String,
    index: u32,
}

#[async_trait]
impl Operator for AvailClient {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: Vec<u8>,
        _prefix: Option<String>,
    ) -> anyhow::Result<()> {
        let submit_url = format!("{}/{}", self.endpoint, SUBMIT_PATH);
        let data = general_purpose::STANDARD.encode(&segment_bytes);
        let max_retries = self.max_retries;
        let mut retries = 0;
        let mut retry_delay = Duration::from_millis(1000);

        loop {
            let response = self
                .client
                .post(&submit_url)
                .header("Content-Type", "application/json")
                .body(json!({ "data": data }).to_string())
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => {
                    let submit_response: AvailSubmitResponse = response.json().await?;
                    info!(
                        "Submitted segment: {} to Avail, block_number: {}, block_hash: {}, hash: {}, index: {}",
                        segment_id,
                        submit_response.block_number,
                        submit_response.block_hash,
                        submit_response.hash,
                        submit_response.index,
                    );
                    return Ok(());
                }
                StatusCode::NOT_FOUND => {
                    return Err(anyhow!(
                        "App mode not active or signing key not configured."
                    ))
                }
                _ => {
                    if retries < max_retries {
                        retries += 1;
                        sleep(retry_delay).await;
                        retry_delay *= 2; // Exponential backoff
                        warn!(
                            "Failed to submit data, retrying after {}ms, attempt: {}",
                            retry_delay.as_millis(),
                            retries
                        );
                    } else {
                        return Err(anyhow!(
                            "Failed to submit data after {} attempts: {}",
                            retries,
                            response.status()
                        ));
                    }
                }
            }
        }
    }
}
