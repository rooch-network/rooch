// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    api::{eth_api::EthAPIServer, RoochRpcModule},
    service::RpcService,
};
use ethers::types::Bytes;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use rooch_types::{
    transaction::{ethereum::EthereumTransaction, AbstractTransaction, TypedTransaction},
    H256,
};

pub struct EthServer {
    rpc_service: RpcService,
}

impl EthServer {
    pub fn new(rpc_service: RpcService) -> Self {
        Self { rpc_service }
    }
}

#[async_trait]
impl EthAPIServer for EthServer {
    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<H256> {
        let tx = TypedTransaction::Ethereum(EthereumTransaction::decode(&bytes)?);
        let hash = tx.hash();
        let _output = self.rpc_service.execute_tx(tx).await?;
        Ok(hash)
    }
}

impl RoochRpcModule for EthServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
