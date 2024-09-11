// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Error, Result};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::metadata::Metadata;
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use moveos_types::access_path::AccessPath;
use moveos_types::h256::H256;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::account::Account;
use moveos_types::moveos_std::move_module::MoveModule;
use moveos_types::moveos_std::object::{ObjectID, ObjectMeta, RawField};
use moveos_types::state::{FieldKey, MoveType, ObjectState};
use moveos_types::state_resolver::{StateKV, StateResolver, StatelessResolver};
use moveos_types::state_root_hash::StateRootHash;
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
                .max_response_size(2 << 30)
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
                let mut states = self
                    .rooch
                    .get_states(AccessPath::module(id), StateRootHash::empty())
                    .await?;
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

#[derive(Clone)]
pub struct ClientResolver {
    root: ObjectMeta,
    client: Client,
}

impl ClientResolver {
    pub fn new(client: Client, root: ObjectMeta) -> Self {
        Self { root, client }
    }
}

impl ResourceResolver for ClientResolver {
    fn get_resource_with_metadata(
        &self,
        address: &AccountAddress,
        resource_tag: &StructTag,
        _metadata: &[Metadata],
    ) -> std::result::Result<(Option<Vec<u8>>, usize), Error> {
        let account_object_id = Account::account_object_id(*address);

        let key = FieldKey::derive_resource_key(resource_tag);
        let result = self
            .get_field(&account_object_id, &key)?
            .map(|s| {
                ensure!(
                    s.match_dynamic_field_type(MoveString::type_tag(), resource_tag.clone().into()),
                    "Resource type mismatch, expected field value type: {:?}, actual: {:?}",
                    resource_tag,
                    s.object_type()
                );
                let field = RawField::parse_resource_field(&s.value, resource_tag.clone().into())?;
                Ok(field.value)
            })
            .transpose();

        match result {
            Ok(opt) => {
                if let Some(data) = opt {
                    Ok((Some(data), 0))
                } else {
                    Ok((None, 0))
                }
            }
            Err(err) => Err(err),
        }
    }
}

impl ModuleResolver for ClientResolver {
    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        vec![]
    }

    fn get_module(&self, id: &ModuleId) -> std::result::Result<Option<Vec<u8>>, Error> {
        (&self.client).get_module(id)
    }
}

impl StatelessResolver for ClientResolver {
    fn get_field_at(&self, state_root: H256, key: &FieldKey) -> Result<Option<ObjectState>, Error> {
        tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                let access_path = AccessPath::object(ObjectID::new(key.0));
                let mut object_state_view_list = self
                    .client
                    .rooch
                    .get_states(
                        access_path,
                        StateRootHash::new(hex::encode(state_root.0.as_slice()).as_str()),
                    )
                    .await?;
                Ok(object_state_view_list.pop().flatten().map(|state_view| {
                    let v: ObjectState = state_view.into();
                    v
                }))
            })
        })
    }

    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                let object_id = ObjectID::new(state_root.0);
                let field_cursor = cursor.map(|field_key| field_key.to_hex_literal());
                let fields_states = self
                    .client
                    .rooch
                    .list_field_states(object_id.into(), field_cursor, Some(limit as u64), None)
                    .await?;
                Ok(fields_states
                    .data
                    .iter()
                    .map(|item| StateKV::from((item.field_key.into(), item.state.clone().into())))
                    .collect())
            })
        })
    }
}

impl StateResolver for ClientResolver {
    fn root(&self) -> &ObjectMeta {
        &self.root
    }
}
