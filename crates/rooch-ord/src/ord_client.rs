// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use reqwest::{header, Client, StatusCode};
use rooch_types::bitcoin::ord::{InscriptionID, SatPoint};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tracing::debug;
use tracing::warn;

pub use ordinals::{Charm, Rune, Sat, SpacedRune};

pub struct OrdClient {
    ord_rpc_url: String,
    http_client: Client,
    max_retries: u32,
    retry_delay: Duration,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Inscriptions {
    pub ids: Vec<InscriptionID>,
    pub more: bool,
    pub page_index: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct InscriptionInfo {
    pub address: Option<String>,
    pub charms: Vec<Charm>,
    pub children: Vec<InscriptionID>,
    pub content_length: Option<usize>,
    pub content_type: Option<String>,
    pub effective_content_type: Option<String>,
    pub fee: u64,
    pub height: u32,
    pub id: InscriptionID,
    pub next: Option<InscriptionID>,
    pub number: i32,
    pub parents: Vec<InscriptionID>,
    pub previous: Option<InscriptionID>,
    pub rune: Option<SpacedRune>,
    pub sat: Option<Sat>,
    pub satpoint: SatPoint,
    pub timestamp: i64,
    pub value: Option<u64>,
}

impl OrdClient {
    pub fn new(ord_rpc_url: String) -> Self {
        let http_client = Client::new();
        Self {
            ord_rpc_url,
            http_client,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }

    async fn retry<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;
        for _ in 0..self.max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if Self::is_network_error(&e) => {
                    warn!("Ord client network error: {:?}, and retry.", e);
                    last_error = Some(e);
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Max retries reached")))
    }

    fn is_network_error(error: &Error) -> bool {
        if let Some(e) = error.downcast_ref::<reqwest::Error>() {
            e.is_connect()
                || e.is_timeout()
                || e.status()
                    .map(|status| status.is_server_error())
                    .unwrap_or(false)
        } else {
            false
        }
    }

    pub async fn get_inscriptions_by_block(&self, height: u64) -> Result<Vec<InscriptionID>> {
        let path = format!("inscriptions/block/{}/0", height);
        let mut result = Vec::new();
        let mut inscriptions: Inscriptions = self.get(path.as_str()).await?.unwrap_or_default();
        result.extend(inscriptions.ids);
        while inscriptions.more {
            let path = format!(
                "inscriptions/block/{}/{}",
                height,
                inscriptions.page_index + 1
            );
            inscriptions = self.get(path.as_str()).await?.unwrap_or_default();
            result.extend(inscriptions.ids);
        }
        Ok(result)
    }

    pub async fn get_inscription(&self, id: &InscriptionID) -> Result<Option<InscriptionInfo>> {
        let path = format!("inscription/{}", id);
        self.get(path.as_str()).await.map_err(|e| {
            warn!("get_inscription {} error: {:?}", id, e);
            e
        })
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        let url = format!("{}/{}", self.ord_rpc_url, path);
        debug!("GET {}", url);
        self.retry(|| async {
            let resp = self
                .http_client
                .get(&url)
                .header(header::ACCEPT, "application/json")
                .send()
                .await?;

            if resp.status() == StatusCode::NOT_FOUND {
                Ok(None)
            } else {
                let resp = resp.error_for_status()?;
                resp.json::<T>().await.map(Some).map_err(Into::into)
            }
        })
        .await
    }
}
