// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::backend::openda::adapter::OpenDAAdapter;
use anyhow::anyhow;
use async_trait::async_trait;
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::{Client, StatusCode};
use rooch_types::da::segment::SegmentID;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

// small blob size for transaction to get included in a block quickly
pub(crate) const DEFAULT_AVAIL_MAX_SEGMENT_SIZE: u64 = 256 * 1024;
// another mechanism guarantees eventual consistency, ok to retry once
pub(crate) const DEFAULT_AVAIL_MAX_RETRIES: usize = 1;
const MAX_BACKOFF_DELAY: Duration = Duration::from_secs(30);

const MIN_BACKOFF_DELAY: Duration = Duration::from_millis(3000);
const SUBMIT_API_PATH: &str = "v2/submit";

const TURBO_MIN_BACKOFF_DELAY: Duration = Duration::from_millis(500);
const TURBO_SUBMIT_API_PATH: &str = "user/submit_raw_data";

/// Avail client: A turbo and Light
/// Turbo client has higher priority, if not available, use the Light client
#[derive(Clone)]
pub struct AvailFusionAdapter {
    turbo_client: Option<AvailTurboClient>,
    light_client: Option<AvailLightClient>,
}

#[async_trait]
impl OpenDAAdapter for AvailFusionAdapter {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: &[u8],
    ) -> anyhow::Result<()> {
        match &self.turbo_client {
            Some(turbo_client) => {
                match turbo_client.submit_segment(segment_id, segment_bytes).await {
                    Ok(result) => return Ok(result), // No fallback needed
                    Err(error) => {
                        tracing::warn!(
                            "Failed to submit segment to Avail Turbo: {}, trying light_client if available",
                            error
                        );
                    }
                }
            }
            None => {
                // No turbo_client, drop directly to light_client
            }
        }

        // If it reaches here, try light_client if available
        if let Some(light_client) = &self.light_client {
            light_client
                .submit_segment(segment_id, segment_bytes) // Takes ownership here
                .await
        } else {
            Err(anyhow!("Both turbo and light clients are not available"))
        }
    }
}

pub struct AvailFusionClientConfig {
    pub turbo_endpoint: Option<String>,
    pub turbo_auth_token: Option<String>,
    pub light_endpoint: Option<String>,
    pub max_retries: usize,
}

impl AvailFusionClientConfig {
    pub fn from_scheme_config(
        scheme_config: HashMap<String, String>,
        max_retries: usize,
    ) -> anyhow::Result<Self> {
        let turbo_endpoint = scheme_config.get("turbo_endpoint").cloned();
        let turbo_auth_token = scheme_config.get("turbo_auth_token").cloned();
        let light_endpoint = scheme_config.get("light_endpoint").cloned();

        if turbo_endpoint.is_none() && light_endpoint.is_none() {
            return Err(anyhow!("turbo_endpoint or light_endpoint must be provided"));
        }
        if turbo_endpoint.is_some() && turbo_auth_token.is_none() {
            return Err(anyhow!("turbo_auth_token must be provided"));
        }

        Ok(AvailFusionClientConfig {
            turbo_endpoint,
            turbo_auth_token,
            light_endpoint,
            max_retries,
        })
    }

    pub fn build_client(&self) -> anyhow::Result<AvailFusionAdapter> {
        let turbo_client = if let Some(endpoint) = &self.turbo_endpoint {
            Some(AvailTurboClient::new(
                endpoint,
                self.max_retries,
                self.turbo_auth_token.as_ref().unwrap(),
            )?)
        } else {
            None
        };
        let light_client = if let Some(endpoint) = &self.light_endpoint {
            Some(AvailLightClient::new(endpoint, self.max_retries)?)
        } else {
            None
        };

        Ok(AvailFusionAdapter {
            turbo_client,
            light_client,
        })
    }
}

#[derive(Clone)]
pub(crate) struct AvailTurboClient {
    endpoint: String,
    http_client: Client,
    max_retries: usize,
    auth_token: String,
}

impl AvailTurboClient {
    pub(crate) fn new(
        endpoint: &str,
        max_retries: usize,
        auth_token: &str,
    ) -> anyhow::Result<Self> {
        let client = Client::new();

        Ok(AvailTurboClient {
            endpoint: endpoint.to_string(),
            http_client: client,
            max_retries,
            auth_token: auth_token.to_string(),
        })
    }

    async fn handle_success(
        segment_id: SegmentID,
        response: reqwest::Response,
    ) -> anyhow::Result<()> {
        match response.json::<AvailTurboClientSubmitResponse>().await {
            Ok(submit_response) => {
                tracing::info!(
                    "Submitted segment: {} to Avail Turbo, submission_id: {}",
                    segment_id,
                    submit_response.submission_id,
                );
                Ok(())
            }
            Err(json_error) => Err(anyhow!(
                "Failed to parse response JSON for segment {:?}: {:?}",
                segment_id,
                json_error,
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailTurboClientSubmitResponse {
    submission_id: String,
}

#[async_trait]
impl OpenDAAdapter for AvailTurboClient {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: &[u8],
    ) -> anyhow::Result<()> {
        let submit_url = format!("{}/{}", self.endpoint, TURBO_SUBMIT_API_PATH);
        let max_attempts = self.max_retries + 1; // max_attempts = max_retries + first attempt
        let mut attempts = 0;
        let mut retry_delay = TURBO_MIN_BACKOFF_DELAY;

        // token for turbo submit,
        // will support more tokens in the future
        const TOKEN: &str = "avail";

        loop {
            attempts += 1;
            let request = self
                .http_client
                .post(&submit_url)
                .query(&[("token", TOKEN.to_string())])
                .bearer_auth(&self.auth_token)
                .header("Content-Type", "application/json")
                .body(segment_bytes.to_vec());

            let response = request.send().await?;

            if response.status().is_success() {
                return AvailTurboClient::handle_success(segment_id, response).await;
            }

            if response.status().is_server_error() {
                if attempts < max_attempts {
                    tracing::warn!(
                            "Failed to submit segment: {:?} to Avail Turbo: {}, attempts: {}，retrying after {}ms",
                            segment_id,
                            response.status(),
                            attempts,
                            retry_delay.as_millis(),
                        );
                    sleep(retry_delay).await;
                    retry_delay = std::cmp::min(retry_delay * 2, MAX_BACKOFF_DELAY);
                    continue;
                }

                return Err(anyhow!(
                    "Failed to submit segment: {:?} to Avail Turbo: {} after {} attempts",
                    segment_id,
                    response.status(),
                    attempts,
                ));
            }
            return Err(anyhow!(
                "Failed to submit segment: {:?} to Avail Turbo: {}",
                segment_id,
                response.status(),
            ));
        }
    }
}

#[derive(Clone)]
pub struct AvailLightClient {
    endpoint: String,
    http_client: Client,
    max_retries: usize,
}

impl AvailLightClient {
    pub fn new(endpoint: &str, max_retries: usize) -> anyhow::Result<Self> {
        let client = Client::new();

        Ok(AvailLightClient {
            endpoint: endpoint.to_string(),
            http_client: client,
            max_retries,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailLightClientSubmitResponse {
    block_number: u32,
    block_hash: String,
    hash: String,
    index: u32,
}

#[async_trait]
impl OpenDAAdapter for AvailLightClient {
    async fn submit_segment(
        &self,
        segment_id: SegmentID,
        segment_bytes: &[u8],
    ) -> anyhow::Result<()> {
        let submit_url = format!("{}/{}", self.endpoint, SUBMIT_API_PATH);
        let data = general_purpose::STANDARD.encode(segment_bytes);
        let max_attempts = self.max_retries + 1; // max_attempts = max_retries + first attempt
        let mut attempts = 0;
        let mut retry_delay = MIN_BACKOFF_DELAY;

        loop {
            attempts += 1;
            let response = self
                .http_client
                .post(&submit_url)
                .header("Content-Type", "application/json")
                .body(json!({ "data": data }).to_string())
                .send()
                .await?;
            match response.status() {
                StatusCode::OK => {
                    let submit_response: AvailLightClientSubmitResponse = response.json().await?;
                    tracing::info!(
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
                        "App mode not active or signing key not configured for Avail."
                    ))
                }
                _ => {
                    if attempts < max_attempts {
                        tracing::warn!(
                            "Failed to submit segment: {:?} to Avail: {}, attempts: {}，retrying after {}ms",
                            segment_id,
                            response.status(),
                            attempts,
                            retry_delay.as_millis(),
                        );
                        sleep(retry_delay).await;
                        retry_delay *= 3; // Exponential backoff
                    } else {
                        return Err(anyhow!(
                            "Failed to submit segment: {:?} to Avail: {} after {} attempts",
                            segment_id,
                            response.status(),
                            attempts,
                        ));
                    }
                }
            }
        }
    }
}
