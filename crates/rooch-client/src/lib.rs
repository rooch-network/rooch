// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use moveos_types::{access_path::AccessPath, object::ObjectID, transaction::FunctionCall};
use rand::Rng;
use rooch_server::{
    api::rooch_api::RoochAPIClient,
    jsonrpc_types::{
        AnnotatedFunctionReturnValueView, AnnotatedMoveStructView, AnnotatedObjectView,
        AnnotatedStateView, ExecuteTransactionResponseView, StateView, TransactionView,
    },
};
use rooch_types::{address::RoochAddress, transaction::rooch::RoochTransaction, H256};

pub mod client_config;
pub mod wallet_context;
use std::sync::Arc;
use std::time::Duration;

pub struct ClientBuilder {
    request_timeout: Duration,
    max_concurrent_requests: usize,
    ws_url: Option<String>,
}

impl ClientBuilder {
    pub fn request_timeout(mut self, request_timeout: Duration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    pub fn max_concurrent_requests(mut self, max_concurrent_requests: usize) -> Self {
        self.max_concurrent_requests = max_concurrent_requests;
        self
    }

    pub fn ws_url(mut self, url: impl AsRef<str>) -> Self {
        self.ws_url = Some(url.as_ref().to_string());
        self
    }

    pub async fn build(self, http: impl AsRef<str>) -> Result<Client> {
        // TODO: add verison info

        let http_client = HttpClientBuilder::default()
            .max_request_body_size(2 << 30)
            .max_concurrent_requests(self.max_concurrent_requests)
            .request_timeout(self.request_timeout)
            .build(http)?;

        Ok(Client {
            rpc: Arc::new(RpcClient { http: http_client }),
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(60),
            max_concurrent_requests: 256,
            ws_url: None,
        }
    }
}

pub(crate) struct RpcClient {
    http: HttpClient,
}

impl std::fmt::Debug for RpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RPC client. Http: {:?}", self.http)
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    rpc: Arc<RpcClient>,
}

impl Client {
    pub async fn execute_tx(&self, tx: RoochTransaction) -> Result<ExecuteTransactionResponseView> {
        let tx_payload = bcs::to_bytes(&tx)?;
        self.rpc
            .http
            .execute_raw_transaction(tx_payload.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn execute_view_function(
        &self,
        function_call: FunctionCall,
    ) -> Result<Vec<AnnotatedFunctionReturnValueView>> {
        self.rpc
            .http
            .execute_view_function(function_call.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn get_resource(
        &self,
        address: AccountAddress,
        resource_type: StructTag,
    ) -> Result<Option<AnnotatedMoveStructView>> {
        Ok(self
            .rpc
            .http
            .get_resource(address, resource_type.into())
            .await?)
    }

    pub async fn get_object(&self, object_id: ObjectID) -> Result<Option<AnnotatedObjectView>> {
        Ok(self.rpc.http.get_object(object_id).await?)
    }

    pub async fn get_states(&self, access_path: AccessPath) -> Result<Vec<Option<StateView>>> {
        Ok(self.rpc.http.get_states(access_path).await?)
    }

    pub async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> Result<Vec<Option<AnnotatedStateView>>> {
        Ok(self.rpc.http.get_annotated_states(access_path).await?)
    }

    pub async fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<TransactionView>> {
        Ok(self.rpc.http.get_transaction_by_hash(hash).await?)
    }

    pub async fn get_transaction_by_index(
        &self,
        start: u64,
        limit: u64,
    ) -> Result<Vec<TransactionView>> {
        let s = self.rpc.http.get_transaction_by_index(start, limit).await?;
        Ok(s)
    }

    pub async fn get_sequence_number(&self, _sender: RoochAddress) -> Result<u64> {
        //TODO read sequencer_number from state,
        // currently, we just generate a random u64 for workaround
        let mut rng = rand::thread_rng();
        Ok(rng.gen())
    }
}
