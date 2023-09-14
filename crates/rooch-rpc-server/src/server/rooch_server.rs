// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::aggregate_service::AggregateService;
use crate::service::rpc_service::RpcService;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use move_core_types::language_storage::StructTag;
use moveos_types::h256::H256;
use rooch_rpc_api::api::{MAX_RESULT_LIMIT, MAX_RESULT_LIMIT_USIZE};
use rooch_rpc_api::jsonrpc_types::account_view::BalanceInfoView;
use rooch_rpc_api::jsonrpc_types::transaction_view::{TransactionReturn, TransactionReturnView};
use rooch_rpc_api::jsonrpc_types::{
    AccessPathView, AccountAddressView, AnnotatedEventView, AnnotatedStateView, EventFilterView,
    EventPageView, ExecuteTransactionResponseView, FunctionCallView, H256View,
    ListAnnotatedStatesPageView, ListBalanceInfoPageView, ListStatesPageView, StateView, StrView,
    StructTagView, TransactionReturnPageView, TransactionView,
};
use rooch_rpc_api::{api::rooch_api::RoochAPIServer, api::DEFAULT_RESULT_LIMIT};
use rooch_rpc_api::{
    api::{RoochRpcModule, DEFAULT_RESULT_LIMIT_USIZE},
    jsonrpc_types::AnnotatedFunctionResultView,
};
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::transaction::{AbstractTransaction, TypedTransaction};
use std::cmp::min;
use tracing::info;

pub struct RoochServer {
    rpc_service: RpcService,
    aggregate_service: AggregateService,
}

impl RoochServer {
    pub fn new(rpc_service: RpcService, aggregate_service: AggregateService) -> Self {
        Self {
            rpc_service,
            aggregate_service,
        }
    }
}

#[async_trait]
impl RoochAPIServer for RoochServer {
    async fn get_chain_id(&self) -> RpcResult<StrView<u64>> {
        let chain_id = self.rpc_service.get_chain_id();
        Ok(StrView(chain_id))
    }

    async fn send_raw_transaction(&self, payload: StrView<Vec<u8>>) -> RpcResult<H256View> {
        info!("send_raw_transaction payload: {:?}", payload);
        let tx = bcs::from_bytes::<RoochTransaction>(&payload.0).map_err(anyhow::Error::from)?;
        info!("send_raw_transaction tx: {:?}", tx);

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
            limit.unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
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
            limit.unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
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
        let limit_of = min(limit.unwrap_or(DEFAULT_RESULT_LIMIT), MAX_RESULT_LIMIT);
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

    async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256View>,
    ) -> RpcResult<Vec<Option<TransactionView>>> {
        let hashes: Vec<H256> = tx_hashes
            .iter()
            .map(|m| (*m).clone().into())
            .collect::<Vec<_>>();
        let resp = self
            .rpc_service
            .get_transactions_by_hash(hashes)
            .await?
            .iter()
            .map(|tx| tx.clone().map(TransactionView::from))
            .collect();

        Ok(resp)
    }

    async fn get_transactions_by_order(
        &self,
        cursor: Option<u128>,
        limit: Option<u64>,
    ) -> RpcResult<TransactionReturnPageView> {
        let limit_of = limit.unwrap_or(DEFAULT_RESULT_LIMIT);

        let mut tx_sequence_mapping = self
            .rpc_service
            .get_tx_sequence_mapping_by_order(cursor, limit_of + 1)
            .await?;

        let has_next_page = (tx_sequence_mapping.len() as u64) > limit_of;
        tx_sequence_mapping.truncate(limit_of as usize);
        let next_cursor = tx_sequence_mapping
            .last()
            .map_or(cursor, |m| Some(m.clone().tx_order));

        let tx_hashes = tx_sequence_mapping
            .clone()
            .iter()
            .map(|m| m.tx_hash)
            .collect::<Vec<_>>();
        let tx_orders = tx_sequence_mapping
            .iter()
            .map(|m| m.tx_order)
            .collect::<Vec<_>>();
        assert_eq!(tx_hashes.len(), tx_orders.len());

        let transactions = self
            .rpc_service
            .get_transactions_by_hash(tx_hashes.clone())
            .await?;

        let sequence_infos = self
            .rpc_service
            .get_transaction_sequence_infos(tx_orders)
            .await?;

        let execution_infos = self
            .rpc_service
            .get_transaction_execution_infos_by_hash(tx_hashes.clone())
            .await?;

        assert!(
            transactions.len() == sequence_infos.len()
                && transactions.len() == execution_infos.len()
        );
        let mut transaction_returns: Vec<TransactionReturn> = vec![];
        for (index, _tx_hash) in tx_hashes.iter().enumerate() {
            let transaction_result = TransactionReturn {
                transaction: transactions[index].clone().unwrap(),
                sequence_info: sequence_infos[index].clone().unwrap(),
                execution_info: execution_infos[index].clone().unwrap(),
            };
            transaction_returns.push(transaction_result)
        }
        let data = transaction_returns
            .into_iter()
            .map(TransactionReturnView::from)
            .collect::<Vec<_>>();

        Ok(TransactionReturnPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    /// get account balances by AccountAddress
    async fn get_balances(
        &self,
        account_addr: AccountAddressView,
        coin_type: Option<StructTagView>,
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> RpcResult<ListBalanceInfoPageView> {
        let limit_of = min(
            limit.unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor_of = cursor.clone().map(|v| v.0);
        let coin_type: Option<StructTag> = coin_type.map(|type_| type_.into());

        let mut data = self
            .aggregate_service
            .get_balances(account_addr.into(), coin_type, cursor_of, limit_of + 1)
            .await?
            .into_iter()
            .map(|item| item.map(|(key, balance_info)| (key, BalanceInfoView::from(balance_info))))
            .collect::<Vec<_>>();

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);

        let next_cursor = data
            .last()
            .cloned()
            .flatten()
            .map_or(cursor, |(key, _balance_info)| key.map(StrView));

        let result = data
            .into_iter()
            .map(|item| item.map(|(_key, balance_info)| balance_info))
            .collect();

        Ok(ListBalanceInfoPageView {
            data: result,
            next_cursor,
            has_next_page,
        })
    }
}

impl RoochRpcModule for RoochServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
