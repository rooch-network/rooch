// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::account_view::BalanceInfoView;
use crate::jsonrpc_types::transaction_view::TransactionResultView;
use crate::jsonrpc_types::{
    AccessPathView, AccountAddressView, AnnotatedFunctionResultView, AnnotatedStateView,
    EventPageView, ExecuteTransactionResponseView, FunctionCallView, H256View,
    ListAnnotatedStatesPageView, ListBalanceInfoPageView, ListStatesPageView, StateView, StrView,
    StructTagView, TransactionResultPageView,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_open_rpc_macros::open_rpc;

#[open_rpc(namespace = "rooch")]
#[rpc(server, client, namespace = "rooch")]
#[async_trait]
pub trait RoochAPI {
    #[method(name = "getChainID")]
    async fn get_chain_id(&self) -> RpcResult<StrView<u64>>;

    /// Send the signed transaction in bcs hex format
    /// This method does not block waiting for the transaction to be executed.
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, tx_bcs_hex: StrView<Vec<u8>>) -> RpcResult<H256View>;

    /// Send the signed transaction in bcs hex format
    /// This method blocks waiting for the transaction to be executed.
    #[method(name = "executeRawTransaction")]
    async fn execute_raw_transaction(
        &self,
        tx_bcs_hex: StrView<Vec<u8>>,
    ) -> RpcResult<ExecuteTransactionResponseView>;

    /// Execute a read-only function call
    /// The function do not change the state of Application
    #[method(name = "executeViewFunction")]
    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<AnnotatedFunctionResultView>;

    /// Get the states by access_path
    #[method(name = "getStates")]
    async fn get_states(&self, access_path: AccessPathView) -> RpcResult<Vec<Option<StateView>>>;

    /// Get the annotated states by access_path
    /// The annotated states include the decoded move value of the state
    #[method(name = "getAnnotatedStates")]
    async fn get_annotated_states(
        &self,
        access_path: AccessPathView,
    ) -> RpcResult<Vec<Option<AnnotatedStateView>>>;

    /// List the states by access_path
    #[method(name = "listStates")]
    async fn list_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> RpcResult<ListStatesPageView>;

    /// List the annotated states by access_path
    /// The annotated states include the decoded move value of the state
    #[method(name = "listAnnotatedStates")]
    async fn list_annotated_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> RpcResult<ListAnnotatedStatesPageView>;

    /// Get the events by event handle id
    #[method(name = "getEventsByEventHandle")]
    async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<u64>,
        limit: Option<u64>,
    ) -> RpcResult<EventPageView>;

    #[method(name = "getTransactionsByHash")]
    async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256View>,
    ) -> RpcResult<Vec<Option<TransactionResultView>>>;

    #[method(name = "getTransactionsByOrder")]
    async fn get_transactions_by_order(
        &self,
        cursor: Option<u128>,
        limit: Option<u64>,
    ) -> RpcResult<TransactionResultPageView>;

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
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> RpcResult<ListBalanceInfoPageView>;
}
