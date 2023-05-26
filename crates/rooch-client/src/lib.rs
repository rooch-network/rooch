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
use moveos::moveos::TransactionOutput;
use moveos_types::object::ObjectID;
use moveos_types::transaction::ViewPayload;
use rand::Rng;
use rooch_common::config::{rooch_config_path, PersistedConfig, RoochConfig};
use rooch_server::api::rooch_api::RoochAPIClient;
use rooch_types::{address::RoochAddress, transaction::rooch::RoochTransaction};

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
        let config: RoochConfig = PersistedConfig::read(rooch_config_path()?.as_path())?;

        let url = match self.rpc.clone() {
            Some(url) => url,
            None => config.server.unwrap().url(false),
        };
        http_client(url)
    }

    pub async fn execute_tx(&self, tx: RoochTransaction) -> Result<TransactionOutput> {
        let txn_payload = bcs::to_bytes(&tx)?;
        self.get_client()?
            .execute_raw_transaction(txn_payload.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn view(&self, payload: ViewPayload) -> Result<Vec<serde_json::Value>> {
        let payload = bcs::to_bytes(&payload)?;
        self.get_client()?
            .view(payload)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> Result<Option<String>> {
        Ok(self
            .get_client()?
            .resource(address, module, resource, type_args)
            .await?)
    }

    pub async fn object(&self, object_id: ObjectID) -> Result<Option<String>> {
        Ok(self.get_client()?.object(object_id).await?)
    }

    pub async fn get_sequence_number(&self, _sender: RoochAddress) -> Result<u64> {
        //TODO read sequencer_number from state,
        // currently, we just generate a random u64 for workaround
        let mut rng = rand::thread_rng();
        Ok(rng.gen())
    }
}
