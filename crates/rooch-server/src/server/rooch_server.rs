// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::api::RoochRpcModule;
use crate::jsonrpc_types::{
    AccessPathView, AnnotatedEventView, AnnotatedFunctionReturnValueView, AnnotatedStateView,
    EventFilterView, EventPageView, ExecuteTransactionResponseView, FunctionCallView,
    RoochH256View, StateView, StrView, StructTagView, TransactionView,
};
use crate::service::RpcService;
use crate::{api::rooch_api::RoochAPIServer, api::MAX_RESULT_LIMIT};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::transaction::{AbstractTransaction, TypedTransaction};

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
    async fn send_raw_transaction(&self, payload: StrView<Vec<u8>>) -> RpcResult<RoochH256View> {
        let tx = bcs::from_bytes::<RoochTransaction>(&payload.0).map_err(anyhow::Error::from)?;
        let hash = tx.tx_hash();
        self.rpc_service
            .quene_tx(TypedTransaction::Rooch(tx))
            .await?;
        Ok(hash.into())
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

    async fn get_states(&self, access_path: AccessPathView) -> RpcResult<Vec<Option<StateView>>> {
        Ok(self
            .rpc_service
            .get_states(access_path.into())
            .await?
            .into_iter()
            .map(|s| s.map(StateView::from))
            .collect())
    }

    async fn get_annotated_states(
        &self,
        access_path: AccessPathView,
    ) -> RpcResult<Vec<Option<AnnotatedStateView>>> {
        Ok(self
            .rpc_service
            .get_annotated_states(access_path.into())
            .await?
            .into_iter()
            .map(|s| s.map(AnnotatedStateView::from))
            .collect())
    }

    async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<u64>,
        limit: Option<u64>,
    ) -> RpcResult<EventPageView> {
        // NOTE: fetch one more object to check if there is next page
        let u_limit = limit.unwrap_or(MAX_RESULT_LIMIT);
        let mut result: Vec<Option<AnnotatedEventView>> = self
            .rpc_service
            .get_events_by_event_handle(event_handle_type.into(), cursor, u_limit + 1)
            .await?
            .into_iter()
            .map(|event| event.map(AnnotatedEventView::from))
            .collect();

        let has_next_page = (result.len() as u64) > u_limit;
        result.truncate(u_limit as usize);
        let next_cursor = result.last().map_or(cursor, |event| {
            Some(event.clone().unwrap().event.event_id.event_seq)
        });

        Ok(EventPageView {
            data: result,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_events(
        &self,
        filter: EventFilterView,
    ) -> RpcResult<Vec<Option<AnnotatedEventView>>> {
        Ok(self
            .rpc_service
            .get_events(filter.into())
            .await?
            .into_iter()
            .map(|event| event.map(AnnotatedEventView::from))
            .collect())
    }

    async fn get_transaction_by_hash(
        &self,
        hash: RoochH256View,
    ) -> RpcResult<Option<TransactionView>> {
        let resp = self
            .rpc_service
            .get_transaction_by_hash(hash.into())
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
