// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::api::RoochRpcModule;
use crate::jsonrpc_types::{
    AnnotatedEventView, AnnotatedFunctionReturnValueView, AnnotatedStateView, EventPage,
    ExecuteTransactionResponseView, FunctionCallView, StateView, StrView, StructTagView,
    TransactionView,
};
use crate::service::RpcService;
use crate::{api::rooch_api::RoochAPIServer, api::MAX_RESULT_LIMIT};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use moveos_types::access_path::AccessPath;
use moveos_types::event_filter::EventFilter;
use moveos_types::transaction::AuthenticatableTransaction;
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
        event_handle_type: StructTagView,
        cursor: Option<u64>,
        limit: Option<u64>,
    ) -> RpcResult<EventPage> {
        // NOTE: fetch one more object to check if there is next page
        let u_limit = limit.unwrap_or(MAX_RESULT_LIMIT);
        let mut result: Vec<Option<AnnotatedEventView>> = self
            .rpc_service
            .get_events_by_event_handle(event_handle_type.into(), cursor.unwrap_or(0), u_limit + 1)
            .await?
            .into_iter()
            .map(|event| event.map(AnnotatedEventView::from))
            .collect();

        let has_next_page = (result.len() as u64) > u_limit;
        let next_cursor = result.last().map_or(cursor, |event| {
            Some(event.clone().unwrap().event.event_id.event_seq)
        });
        result.truncate(u_limit as usize);

        Ok(EventPage {
            data: result,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_events(&self, filter: EventFilter) -> RpcResult<Vec<Option<AnnotatedEventView>>> {
        let data = self
            .rpc_service
            .get_events(filter)
            .await?
            .into_iter()
            .map(|event| event.map(AnnotatedEventView::from))
            .collect();

        // let result = Events

        // let result: Vec<Option<AnnotatedEventView>> = Vec::new();
        Ok(data)
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
