// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::api::rooch_api::RoochAPIServer;
use crate::api::RoochRpcModule;
use crate::jsonrpc_types::{
    AnnotatedFunctionReturnValueView, AnnotatedStateView, EventView,
    ExecuteTransactionResponseView, FunctionCallView, StateView, StrView, TransactionView,
};
use crate::service::RpcService;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use moveos_types::access_path::AccessPath;
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
    ) -> RpcResult<ExecuteTransactionResponseView> {
        let tx = bcs::from_bytes::<RoochTransaction>(&payload.0).map_err(anyhow::Error::from)?;
        Ok(self
            .rpc_service
            .execute_tx(TypedTransaction::Rooch(tx))
            .await?
            .into())
    }

    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<Vec<AnnotatedFunctionReturnValueView>> {
        Ok(self
            .rpc_service
            .execute_view_function(function_call.into())
            .await?
            .into_iter()
            .map(AnnotatedFunctionReturnValueView::from)
            .collect())
    }

    async fn get_states(&self, access_path: AccessPath) -> RpcResult<Vec<Option<StateView>>> {
        Ok(self
            .rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .map(|s| s.map(StateView::from))
            .collect())
    }

    async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> RpcResult<Vec<Option<AnnotatedStateView>>> {
        Ok(self
            .rpc_service
            .get_annotated_states(access_path)
            .await?
            .into_iter()
            .map(|s| s.map(AnnotatedStateView::from))
            .collect())
    }

    async fn get_events_by_event_handle(
        &self,
        event_handle_id: ObjectID,
    ) -> RpcResult<Option<Vec<EventView>>> {
        let mut result: Vec<EventView> = Vec::new();
        let events = self
            .rpc_service
            .get_events_by_event_handle(event_handle_id)
            .await?;
        if Option::is_some(&events) {
            for ev in events
                .unwrap()
                .iter()
                .enumerate()
                .map(|(_i, event)| EventView::from(event.clone()))
                .collect::<Vec<_>>()
            {
                result.push(ev);
            }
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn get_events(&self, filter: EventFilter) -> RpcResult<Option<Vec<EventView>>> {
        let mut result: Vec<EventView> = Vec::new();
        let events = self.rpc_service.get_events(filter).await?;
        if Option::is_some(&events) {
            for ev in events
                .unwrap()
                .iter()
                .enumerate()
                .map(|(_i, event)| EventView::from(event.clone()))
                .collect::<Vec<_>>()
            {
                result.push(ev);
            }
        }

        if !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<Option<TransactionView>> {
        let resp = self
            .rpc_service
            .get_transaction_by_hash(hash)
            .await?
            .map(Into::into);
        Ok(resp)
    }

    async fn get_transaction_by_index(
        &self,
        start: u64,
        limit: u64,
    ) -> RpcResult<Vec<TransactionView>> {
        let resp = self
            .rpc_service
            .get_transaction_by_index(start, limit)
            .await?
            .iter()
            .map(|s| TransactionView::from(s.clone()))
            .collect();

        Ok(resp)
    }
}

impl RoochRpcModule for RoochServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
