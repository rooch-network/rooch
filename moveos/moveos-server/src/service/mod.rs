// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::proxy::ServerProxy;
use crate::response::JsonResponse;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};

// Define a rpc server api
#[rpc(server, client)]
pub trait RpcService {
    #[method(name = "echo")]
    async fn echo(&self, msg: String) -> RpcResult<JsonResponse<String>>;

    // TODO: add suitable response type.
    #[method(name = "submit_txn")]
    async fn submit_txn(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<String>>;

    // TODO: add suitable response type.
    #[method(name = "view")]
    async fn view(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<Vec<serde_json::Value>>>;

    #[method(name = "resource")]
    async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> RpcResult<JsonResponse<String>>;
}

pub struct RoochServer {
    manager: ServerProxy,
}

impl RoochServer {
    pub fn new(manager: ServerProxy) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl RpcServiceServer for RoochServer {
    async fn echo(&self, msg: String) -> RpcResult<JsonResponse<String>> {
        let resp = self.manager.echo(msg).await?;
        Ok(JsonResponse::ok(resp))
    }

    async fn submit_txn(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<String>> {
        let resp = self.manager.submit_txn(payload).await?;
        Ok(JsonResponse::ok(resp))
    }

    async fn view(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<Vec<serde_json::Value>>> {
        let output_values = self.manager.view(payload).await?;
        println!("Output values: {:?}", output_values.clone());
        let mut resp = vec![];
        for v in output_values {
            resp.push(serde_json::to_value(v)?);
        }
        // println!("{}", resp);
        Ok(JsonResponse::ok(resp))
    }

    async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> RpcResult<JsonResponse<String>> {
        let resp = self
            .manager
            .resource(address, &module, &resource, type_args)
            .await?;
        Ok(JsonResponse::ok(resp))
    }
}
