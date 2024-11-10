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
use tracing::info;

const SUBMIT_PATH: &str = "v2/submit";

pub(crate) struct AvailClient {
    endpoint: String,
    client: Client,
}

impl AvailClient {
    pub(crate) fn new(endpoint: &str) -> anyhow::Result<Self> {
        let client = Client::new();

        Ok(AvailClient {
            endpoint: endpoint.to_string(),
            client,
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
                Ok(())
            }
            StatusCode::NOT_FOUND => Err(anyhow!(
                "App mode not active or signing key not configured."
            )),
            _ => Err(anyhow!("Failed to submit data: {}", response.status())),
        }
    }
}
