// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use move_core_types::language_storage::ModuleId;
use move_core_types::metadata::Metadata;
use move_core_types::resolver::ModuleResolver;
use moveos_types::access_path::AccessPath;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::move_module::MoveModule;
use moveos_types::state::ObjectState;
use moveos_types::{
    function_return_value::FunctionResult, module_binding::MoveFunctionCaller,
    moveos_std::tx_context::TxContext, transaction::FunctionCall,
};
use rooch_client::RoochRpcClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;

pub mod client_config;
pub mod rooch_client;
pub mod wallet_context;

pub struct ClientBuilder {
    request_timeout: Duration,
    ws_url: Option<String>,
}

impl ClientBuilder {
    pub fn request_timeout(mut self, request_timeout: Duration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    pub fn ws_url(mut self, url: impl AsRef<str>) -> Self {
        self.ws_url = Some(url.as_ref().to_string());
        self
    }

    pub async fn build(self, http: impl AsRef<str>) -> Result<Client> {
        // TODO: add verison info

        let http_client = Arc::new(
            HttpClientBuilder::default()
                .max_request_size(2 << 30)
                .request_timeout(self.request_timeout)
                .build(http)?,
        );

        Ok(Client {
            http: http_client.clone(),
            rooch: RoochRpcClient::new(http_client.clone()),
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(60),
            ws_url: None,
        }
    }
}

#[derive(Clone)]
pub struct Client {
    http: Arc<HttpClient>,
    pub rooch: RoochRpcClient,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RPC client. Http: {:?}", self.http)
    }
}

impl Client {
    pub async fn request(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        Ok(self.http.request(method, params).await?)
    }
}

impl MoveFunctionCaller for Client {
    fn call_function(
        &self,
        _ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        let function_result =
            futures::executor::block_on(self.rooch.execute_view_function(function_call))?;
        function_result.try_into()
    }
}

impl ModuleResolver for &Client {
    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        Vec::new()
    }

    fn get_module(&self, id: &ModuleId) -> Result<Option<Vec<u8>>> {
        tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                let mut states = self.rooch.get_states(AccessPath::module(id)).await?;
                states
                    .pop()
                    .flatten()
                    .map(|state_view| {
                        let state = ObjectState::from(state_view);
                        let module = state.value_as_df::<MoveString, MoveModule>()?;
                        Ok(module.value.byte_codes)
                    })
                    .transpose()
            })
        })
    }
}
