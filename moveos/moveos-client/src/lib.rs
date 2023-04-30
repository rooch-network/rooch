use anyhow::Result;
use clap::Parser;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use moveos::types::transaction::{SimpleTransaction, ViewPayload};
use moveos_common::config::load_config;
use moveos_server::service::RpcServiceClient;
// |use tokio::time::Duration;

#[derive(Clone, Debug, Parser)]
pub struct Client {
    #[clap(long)]
    rpc: Option<String>,
}

pub fn http_client(url: impl AsRef<str>) -> Result<HttpClient> {
    let client = HttpClientBuilder::default().build(url)?;
    Ok(client)
}

impl Client {
    pub fn connect(&self) -> Result<()> {
        // if self.http_client.is_some() {
        //     return Ok(());
        // }
        // let url = match self.rpc.clone() {
        //     Some(url) => url,
        //     None => load_config()?.server.url(false),
        // };
        // self.http_client = Some(HttpClientBuilder::default().build(url)?);
        Ok(())
    }

    fn get_client(&self) -> Result<HttpClient> {
        let url = match self.rpc.clone() {
            Some(url) => url,
            None => load_config()?.server.url(false),
        };
        http_client(url)
    }

    pub async fn submit_txn(&self, txn: SimpleTransaction) -> Result<()> {
        let txn_payload = bcs::to_bytes(&txn)?;
        let resp = self.get_client()?.submit_txn(txn_payload).await?;
        // TODO: parse response.
        println!("{:?}", resp);
        Ok(())
    }

    pub async fn view(&self, payload: ViewPayload) -> Result<()> {
        let payload = bcs::to_bytes(&payload)?;
        let resp = self.get_client()?.view(payload).await?;
        // TODO: parse response.
        println!("{:?}", resp);
        Ok(())
    }
}
