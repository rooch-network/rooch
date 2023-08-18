// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::RpcService;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use moveos_types::h256::H256;
use rooch_rpc_api::jsonrpc_types::{
    AccessPathView, AnnotatedEventView, AnnotatedStateView, EventFilterView, EventPageView,
    ExecuteTransactionResponseView, FunctionCallView, H256View, ListAnnotatedStatesPageView,
    ListStatesPageView, StateView, StrView, StructTagView, TransactionExecutionInfoView,
    TransactionInfoPageView, TransactionView,
};
use rooch_rpc_api::{api::rooch_api::RoochAPIServer, api::MAX_RESULT_LIMIT};
use rooch_rpc_api::{
    api::{RoochRpcModule, MAX_RESULT_LIMIT_USIZE},
    jsonrpc_types::AnnotatedFunctionResultView,
};
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::transaction::{AbstractTransaction, TypedTransaction};
use std::cmp::min;

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
    async fn send_raw_transaction(&self, payload: StrView<Vec<u8>>) -> RpcResult<H256View> {
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
    ) -> RpcResult<AnnotatedFunctionResultView> {
        Ok(self
            .rpc_service
            .execute_view_function(function_call.into())
            .await?
            .into())
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

    async fn list_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> RpcResult<ListStatesPageView> {
        let limit_of = min(
            limit.unwrap_or(MAX_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor_of = cursor.clone().map(|v| v.0);
        let mut data: Vec<Option<(Vec<u8>, StateView)>> = self
            .rpc_service
            .list_states(access_path.into(), cursor_of, limit_of + 1)
            .await?
            .into_iter()
            .map(|item| item.map(|(key, state)| (key, StateView::from(state))))
            .collect::<Vec<_>>();

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().map_or(cursor, |item| {
            item.clone().map(|(key, _state)| StrView(key))
        });
        let result = data
            .into_iter()
            .map(|item| item.map(|(_key, state)| state))
            .collect();

        Ok(ListStatesPageView {
            data: result,
            next_cursor,
            has_next_page,
        })
    }

    async fn list_annotated_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> RpcResult<ListAnnotatedStatesPageView> {
        let limit_of = min(
            limit.unwrap_or(MAX_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor_of = cursor.clone().map(|v| v.0);
        let mut data: Vec<Option<(Vec<u8>, AnnotatedStateView)>> = self
            .rpc_service
            .list_annotated_states(access_path.into(), cursor_of, limit_of + 1)
            .await?
            .into_iter()
            .map(|item| item.map(|(key, state)| (key, AnnotatedStateView::from(state))))
            .collect::<Vec<_>>();

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().map_or(cursor, |item| {
            item.clone().map(|(key, _state)| StrView(key))
        });
        let result = data
            .into_iter()
            .map(|item| item.map(|(_key, state)| state))
            .collect();

        Ok(ListAnnotatedStatesPageView {
            data: result,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<u64>,
        limit: Option<u64>,
    ) -> RpcResult<EventPageView> {
        // NOTE: fetch one more object to check if there is next page
        let limit_of = min(limit.unwrap_or(MAX_RESULT_LIMIT), MAX_RESULT_LIMIT);
        let mut data: Vec<Option<AnnotatedEventView>> = self
            .rpc_service
            .get_events_by_event_handle(event_handle_type.into(), cursor, limit_of + 1)
            .await?
            .into_iter()
            .map(|event| event.map(AnnotatedEventView::from))
            .collect();

        let has_next_page = (data.len() as u64) > limit_of;
        data.truncate(limit_of as usize);
        let next_cursor = data.last().map_or(cursor, |event| {
            Some(event.clone().unwrap().event.event_id.event_seq)
        });

        Ok(EventPageView {
            data,
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

    async fn get_transaction_by_hash(&self, hash: H256View) -> RpcResult<Option<TransactionView>> {
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

    async fn get_transaction_infos_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: Option<u64>,
    ) -> RpcResult<TransactionInfoPageView> {
        // NOTE: fetch one more object to check if there is next page
        let limit_of = limit.unwrap_or(MAX_RESULT_LIMIT);

        let mut tx_seq_mapping = self
            .rpc_service
            .get_tx_seq_mapping_by_tx_order(cursor, limit_of + 1)
            .await?;

        let has_next_page = (tx_seq_mapping.len() as u64) > limit_of;
        tx_seq_mapping.truncate(limit_of as usize);
        let next_cursor = tx_seq_mapping
            .last()
            .map_or(cursor, |m| Some(m.clone().tx_order));

        let tx_hashes = tx_seq_mapping.iter().map(|m| m.tx_hash).collect::<Vec<_>>();

        let result = self
            .rpc_service
            .get_transaction_infos_by_tx_hash(tx_hashes)
            .await?
            .into_iter()
            .map(|tx_info| tx_info.map(TransactionExecutionInfoView::from))
            .collect();

        Ok(TransactionInfoPageView {
            data: result,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_transaction_infos_by_tx_hash(
        &self,
        tx_hashes: Vec<H256View>,
    ) -> RpcResult<Vec<Option<TransactionExecutionInfoView>>> {
        let hashes: Vec<H256> = tx_hashes
            .iter()
            .map(|m| (*m).clone().into())
            .collect::<Vec<_>>();

        let result = self
            .rpc_service
            .get_transaction_infos_by_tx_hash(hashes)
            .await?
            .into_iter()
            .map(|tx_info| tx_info.map(TransactionExecutionInfoView::from))
            .collect();

        Ok(result)
    }
}

impl RoochRpcModule for RoochServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
