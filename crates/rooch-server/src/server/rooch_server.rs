// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::api::RoochRpcModule;
use crate::jsonrpc_types::{
    AnnotatedMoveStructView, EventView, FunctionCallView, StrView, StructTagView,
};
use crate::service::RpcService;
use crate::{api::rooch_api::RoochAPIServer, jsonrpc_types::AnnotatedObjectView};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use move_core_types::account_address::AccountAddress;
use moveos::moveos::TransactionOutput;
use moveos_types::event_filter::EventFilter;
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
    async fn send_raw_transaction(&self, payload: StrView<Vec<u8>>) -> RpcResult<H256> {
        let tx = bcs::from_bytes::<RoochTransaction>(&payload.0).map_err(anyhow::Error::from)?;
        let hash = tx.tx_hash();
        self.rpc_service
            .quene_tx(TypedTransaction::Rooch(tx))
            .await?;
        Ok(hash)
    }

    async fn execute_raw_transaction(
        &self,
        payload: StrView<Vec<u8>>,
    ) -> RpcResult<TransactionOutput> {
        let tx = bcs::from_bytes::<RoochTransaction>(&payload.0).map_err(anyhow::Error::from)?;
        Ok(self
            .rpc_service
            .execute_tx(TypedTransaction::Rooch(tx))
            .await?)
    }

    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<Vec<serde_json::Value>> {
        Ok(self
            .rpc_service
            .execute_view_function(function_call.into())
            .await?)
    }

    async fn get_resource(
        &self,
        address: AccountAddress,
        resource_type: StructTagView,
    ) -> RpcResult<Option<AnnotatedMoveStructView>> {
        Ok(self
            .rpc_service
            .get_resource(address, resource_type.into())
            .await?
            .map(Into::into))
    }

    async fn get_object(&self, object_id: ObjectID) -> RpcResult<Option<AnnotatedObjectView>> {
        Ok(self.rpc_service.object(object_id).await?.map(Into::into))
    }

    async fn get_events_by_tx_hash(&self, tx_hash: H256) -> RpcResult<Option<Vec<EventView>>> {
        let mut result: Vec<EventView> = Vec::new();
        for ev in self
            .rpc_service
            .get_events_by_tx_hash(tx_hash)
            .await?
            .unwrap()
            .iter()
            .enumerate()
            .map(|(_i, event)| EventView::from(event.clone()))
            .collect::<Vec<_>>()
        {
            result.push(ev);
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn get_events(&self, filter: EventFilter) -> RpcResult<Option<Vec<EventView>>> {
        let mut result: Vec<EventView> = Vec::new();
        for ev in self
            .rpc_service
            .get_events(filter)
            .await?
            .unwrap()
            .iter()
            .enumerate()
            .map(|(_i, event)| EventView::from(event.clone()))
            .collect::<Vec<_>>()
        {
            result.push(ev);
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

impl RoochRpcModule for RoochServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
