// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
mod client;
#[cfg(test)]
mod test;

use anyhow::Result;
use clap::Parser;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use moveos_common::config::load_config;
use moveos_types::object::ObjectID;
use moveos_types::transaction::ViewPayload;
use rand::Rng;
use rooch_server::service::RpcServiceClient;
use rooch_types::{address::RoochAddress, transaction::rooch::RoochTransaction};
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

    pub async fn submit_txn(&self, tx: RoochTransaction) -> Result<()> {
        let txn_payload = bcs::to_bytes(&tx)?;
        let resp = self.get_client()?.submit_txn(txn_payload).await?;
        // TODO: return the transaction output to caller.
        match resp.try_into_inner()? {
            Some(v) => println!("{:?}", v),
            None => println!("{:?}", serde_json::Value::Null),
        };
        Ok(())
    }

    pub async fn view(&self, payload: ViewPayload) -> Result<()> {
        let payload = bcs::to_bytes(&payload)?;
        let resp = self.get_client()?.view(payload).await?;
        // TODO: parse response.
        match resp.try_into_inner()? {
            Some(v) => println!("{:?}", v),
            None => println!("{:?}", serde_json::Value::Null),
        };
        Ok(())
    }

    pub async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> Result<Option<String>> {
        let resp = self
            .get_client()?
            .resource(address, module, resource, type_args)
            .await?;
        resp.try_into_inner()
    }

    pub async fn object(&self, object_id: ObjectID) -> Result<Option<String>> {
        let resp = self.get_client()?.object(object_id).await?;
        resp.try_into_inner()
    }

    pub async fn get_sequence_number(&self, _sender: RoochAddress) -> Result<u64> {
        //TODO read sequencer_number from state,
        // currently, we just generate a random u64 for workaround
        let mut rng = rand::thread_rng();
        Ok(rng.gen())
    }
}
