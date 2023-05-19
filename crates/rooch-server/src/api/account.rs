// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::api::RoochRpcModule;
use crate::jsonrpc_types::coin::Balance;
use crate::response::JsonResponse;
use async_trait::async_trait;
use jsonrpsee::RpcModule;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use move_core_types::account_address::AccountAddress;
use moveos_types::move_types::StructId;
use rooch_executor::proxy::ExecutorProxy;
use tracing::{info, instrument};

// #[rpc(server, client, namespace = "rooch")]
#[rpc(server, client)]
pub trait AccountApi {
    #[method(name = "accounts.get_balance")]
    async fn get_balance(&self, token: String) -> RpcResult<JsonResponse<Balance>>;

    #[method(name = "accounts.get")]
    async fn get(
        &self,
        address: AccountAddress,
        resource: StructId,
    ) -> RpcResult<JsonResponse<String>>;
}

pub struct AccountServer {
    executor: ExecutorProxy,
}

impl AccountServer {
    pub fn new(executor: ExecutorProxy) -> Self {
        Self { executor }
    }

    #[instrument(skip(self))]
    async fn get_balance(&self, token: String) -> RpcResult<JsonResponse<Balance>> {
        info!("get_balance");

        //TODO Mock get_balance
        Ok(JsonResponse::ok(Balance {
            // coin_type: "Rooch".to_string(),
            coin_type: token,
            total_balance: 1005,
        }))
    }

    #[instrument(skip(self))]
    async fn get(
        &self,
        address: AccountAddress,
        resource: StructId,
    ) -> RpcResult<JsonResponse<String>> {
        let resp = self
            .executor
            .resource(
                address,
                &resource.module_id.clone(),
                &resource.struct_id.clone(),
                Vec::new(),
            )
            .await?;

        //TODO convert MoveResource to Rust Struct
        println!("{:?}", resp);
        Ok(JsonResponse::ok(resp))
    }
}

#[async_trait]
impl AccountApiServer for AccountServer {
    async fn get_balance(&self, token: String) -> RpcResult<JsonResponse<Balance>> {
        Ok(self.get_balance(token).await?)
    }

    async fn get(
        &self,
        address: AccountAddress,
        resource: StructId,
    ) -> RpcResult<JsonResponse<String>> {
        Ok(self.get(address, resource).await?)
    }
}

impl RoochRpcModule for AccountServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
