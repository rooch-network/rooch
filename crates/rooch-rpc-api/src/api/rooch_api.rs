// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::account_view::BalanceInfoView;
use crate::jsonrpc_types::event_view::EventFilterView;
use crate::jsonrpc_types::transaction_view::{TransactionFilterView, TransactionWithInfoView};
use crate::jsonrpc_types::{
    AccessPathView, AccountAddressView, AnnotatedFunctionResultView, BalanceInfoPageView,
    BytesView, EventOptions, EventPageView, ExecuteTransactionResponseView, FunctionCallView,
    GlobalStateFilterView, H256View, IndexerEventPageView, IndexerGlobalStatePageView,
    IndexerTableChangeSetPageView, IndexerTableStatePageView, SimpleKeyStateView, StateOptions,
    StateSyncFilterView, StateView, StatesPageView, StrView, StructTagView, TableStateFilterView,
    TransactionWithInfoPageView,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_open_rpc_macros::open_rpc;
use rooch_types::indexer::event_filter::IndexerEventID;
use rooch_types::indexer::state::IndexerStateID;

#[open_rpc(namespace = "rooch")]
#[rpc(server, client, namespace = "rooch")]
#[async_trait]
pub trait RoochAPI {
    #[method(name = "getChainID")]
    async fn get_chain_id(&self) -> RpcResult<StrView<u64>>;

    /// Send the signed transaction in bcs hex format
    /// This method does not block waiting for the transaction to be executed.
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, tx_bcs_hex: BytesView) -> RpcResult<H256View>;

    /// Send the signed transaction in bcs hex format
    /// This method blocks waiting for the transaction to be executed.
    #[method(name = "executeRawTransaction")]
    async fn execute_raw_transaction(
        &self,
        tx_bcs_hex: BytesView,
    ) -> RpcResult<ExecuteTransactionResponseView>;

    /// Execute a read-only function call
    /// The function do not change the state of Application
    #[method(name = "executeViewFunction")]
    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<AnnotatedFunctionResultView>;

    /// Get the states by access_path
    /// If the StateOptions.decode is true, the state is decoded and the decoded value is returned in the response.
    #[method(name = "getStates")]
    async fn get_states(
        &self,
        access_path: AccessPathView,
        state_option: Option<StateOptions>,
    ) -> RpcResult<Vec<Option<StateView>>>;

    /// List the states by access_path
    /// If the StateOptions.decode is true, the state is decoded and the decoded value is returned in the response.
    #[method(name = "listStates")]
    async fn list_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<SimpleKeyStateView>,
        limit: Option<StrView<usize>>,
        state_option: Option<StateOptions>,
    ) -> RpcResult<StatesPageView>;

    /// Get the events by event handle id
    #[method(name = "getEventsByEventHandle")]
    async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        event_options: Option<EventOptions>,
    ) -> RpcResult<EventPageView>;

    #[method(name = "getTransactionsByHash")]
    async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256View>,
    ) -> RpcResult<Vec<Option<TransactionWithInfoView>>>;

    #[method(name = "getTransactionsByOrder")]
    async fn get_transactions_by_order(
        &self,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
    ) -> RpcResult<TransactionWithInfoPageView>;

    /// get account balance by AccountAddress and CoinType
    #[method(name = "getBalance")]
    async fn get_balance(
        &self,
        account_addr: AccountAddressView,
        coin_type: StructTagView,
    ) -> RpcResult<BalanceInfoView>;

    /// get account balances by AccountAddress
    #[method(name = "getBalances")]
    async fn get_balances(
        &self,
        account_addr: AccountAddressView,
        cursor: Option<SimpleKeyStateView>,
        limit: Option<StrView<usize>>,
    ) -> RpcResult<BalanceInfoPageView>;

    /// Query the transactions indexer by transaction filter
    #[method(name = "queryTransactions")]
    async fn query_transactions(
        &self,
        filter: TransactionFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<TransactionWithInfoPageView>;

    /// Query the events indexer by event filter
    #[method(name = "queryEvents")]
    async fn query_events(
        &self,
        filter: EventFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerEventID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<IndexerEventPageView>;

    /// Query the global states indexer by state filter
    #[method(name = "queryGlobalStates")]
    async fn query_global_states(
        &self,
        filter: GlobalStateFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<IndexerGlobalStatePageView>;

    /// Query the table states indexer by state filter
    #[method(name = "queryTableStates")]
    async fn query_table_states(
        &self,
        filter: TableStateFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<IndexerTableStatePageView>;

    /// Sync state change sets from indexer
    #[method(name = "syncStates")]
    async fn sync_states(
        &self,
        filter: Option<StateSyncFilterView>,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<IndexerTableChangeSetPageView>;
}
