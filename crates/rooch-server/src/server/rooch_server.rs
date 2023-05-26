// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::api::rooch_api::RoochAPIServer;
use crate::api::RoochRpcModule;
use crate::service::RpcService;
use ethers::types::Bytes;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use moveos::moveos::TransactionOutput;
use moveos_types::{object::ObjectID, transaction::AuthenticatableTransaction};
use rooch_types::transaction::TypedTransaction;
use rooch_types::{transaction::rooch::RoochTransaction, H256};

pub struct RoochServer {
    rpc_service: RpcService,
}

impl RoochServer {
    pub fn new(rpc_service: RpcService) -> Self {
        Self { rpc_service }
    }
}

#[async_trait]
impl RoochAPIServer for RoochServer {
    async fn echo(&self, msg: String) -> RpcResult<String> {
        Ok(msg)
    }

    async fn send_raw_transaction(&self, payload: Bytes) -> RpcResult<H256> {
        let tx = bcs::from_bytes::<RoochTransaction>(&payload).map_err(anyhow::Error::from)?;
        let hash = tx.tx_hash();
        self.rpc_service
            .quene_tx(TypedTransaction::Rooch(tx))
            .await?;
        Ok(hash)
    }

    async fn execute_raw_transaction(&self, payload: Bytes) -> RpcResult<TransactionOutput> {
        let tx = bcs::from_bytes::<RoochTransaction>(&payload).map_err(anyhow::Error::from)?;
        Ok(self
            .rpc_service
            .execute_tx(TypedTransaction::Rooch(tx))
            .await?)
    }

    async fn view(&self, payload: Vec<u8>) -> RpcResult<Vec<serde_json::Value>> {
        Ok(self.rpc_service.view(payload).await?)
    }

    async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> RpcResult<Option<String>> {
        Ok(self
            .rpc_service
            .resource(address, module, resource, type_args)
            .await?)
    }

    async fn object(&self, object_id: ObjectID) -> RpcResult<Option<String>> {
        Ok(self.rpc_service.object(object_id).await?)
    }
}

impl RoochRpcModule for RoochServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
