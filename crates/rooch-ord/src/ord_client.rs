// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use reqwest::{header, Client};
use rooch_types::bitcoin::ord::{InscriptionID, SatPoint};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::debug;

pub use ordinals::{Charm, Rune, Sat, SpacedRune};

pub struct OrdClient {
    ord_rpc_url: String,
    http_client: Client,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
        }
    }

    pub async fn get_inscriptions_by_block(&self, height: u64) -> Result<Vec<InscriptionID>> {
        let path = format!("inscriptions/block/{}/0", height);
        let mut result = Vec::new();
        let mut inscriptions: Inscriptions = self.get(path.as_str()).await?;
        result.extend(inscriptions.ids);
        while inscriptions.more {
            let path = format!(
                "inscriptions/block/{}/{}",
                height,
                inscriptions.page_index + 1
            );
            inscriptions = self.get(path.as_str()).await?;
            result.extend(inscriptions.ids);
        }
        Ok(result)
    }

    pub async fn get_inscription(&self, id: &InscriptionID) -> Result<Option<InscriptionInfo>> {
        let path = format!("inscription/{}", id);
        Ok(self.get(path.as_str()).await.ok())
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}/{}", self.ord_rpc_url, path);
        debug!("GET {}", url);
        Ok(self
            .http_client
            .get(url)
            .header(header::ACCEPT, "application/json")
            .send()
            .await?
            .json::<T>()
            .await?)
    }
}

#[tokio::test]
async fn test() {
    let ord_client = OrdClient::new("http://localhost:8080".to_string());
    let inscriptions = ord_client.get_inscriptions_by_block(790964).await.unwrap(); //.http_client.get("http://localhost:8080/inscriptions/block/790964/0").header(header::ACCEPT, "application/json").send().await.unwrap();
                                                                                    //println!("{}",resp.clone().text().await.unwrap());
                                                                                    //let inscriptions = resp.json::<Inscriptions>().await.unwrap();
    println!("{:?}", inscriptions);
    let id = inscriptions[0].clone();
    let inscription = ord_client.get_inscription(&id).await.unwrap();
    println!("{:?}", inscription);
}
