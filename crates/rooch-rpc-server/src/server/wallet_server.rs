// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::rpc_service::RpcService;
use ethers::types::Bytes;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::RpcModule;
use rooch_rpc_api::api::wallet_api::WalletApiServer;
use rooch_rpc_api::api::RoochRpcModule;
use rooch_types::address::RoochAddress;

pub struct WalletServer {
    rpc_service: RpcService,
}

impl WalletServer {
    pub fn new(rpc_service: RpcService) -> Self {
        Self { rpc_service }
    }
}

#[async_trait]
impl WalletApiServer for WalletServer {
    async fn sign(&self, address: RoochAddress, message: Bytes) -> RpcResult<Bytes> {
        Ok(self
            .rpc_service
            .sign(address, message.to_vec())
            .await?
            .into())
    }

    async fn accounts(&self) -> RpcResult<Vec<RoochAddress>> {
        Ok(self.rpc_service.accounts().await?)
    }
}

impl RoochRpcModule for WalletServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
