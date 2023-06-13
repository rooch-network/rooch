// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AccessPathView, AnnotatedEventView, AnnotatedFunctionReturnValueView, AnnotatedStateView,
    EventFilterView, EventPageView, ExecuteTransactionResponseView, FunctionCallView,
    RoochH256View, StateView, StrView, StructTagView, TransactionView,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_open_rpc_macros::open_rpc;

#[open_rpc(namespace = "rooch")]
#[rpc(server, client, namespace = "rooch")]
pub trait RoochAPI {
    /// Send the signed transaction in bcs hex format
    /// This method does not block waiting for the transaction to be executed.
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, tx_bcs_hex: StrView<Vec<u8>>) -> RpcResult<RoochH256View>;

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
    ) -> RpcResult<Vec<AnnotatedFunctionReturnValueView>>;

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

    /// Get the events by event handle id
    #[method(name = "getEventsByEventHandle")]
    async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<u64>,
        limit: Option<u64>,
    ) -> RpcResult<EventPageView>;

    /// Get the events by event filter
    #[method(name = "getEvents")]
    async fn get_events(
        &self,
        filter: EventFilterView,
    ) -> RpcResult<Vec<Option<AnnotatedEventView>>>;

    #[method(name = "getTransactionByHash")]
    async fn get_transaction_by_hash(
        &self,
        hash: RoochH256View,
    ) -> RpcResult<Option<TransactionView>>;

    #[method(name = "getTransactionByIndex")]
    async fn get_transaction_by_index(
        &self,
        start: u64,
        limit: u64,
    ) -> RpcResult<Vec<TransactionView>>;
}
