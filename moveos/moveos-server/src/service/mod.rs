// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::proxy::ServerProxy;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;

use crate::response::JsonResponse;

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
    async fn view(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<String>>;
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

    async fn view(&self, payload: Vec<u8>) -> RpcResult<JsonResponse<String>> {
        let resp = self.manager.view(payload).await?;
        Ok(JsonResponse::ok(resp))
    }
}
