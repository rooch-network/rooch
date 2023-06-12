// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AnnotatedEventView, AnnotatedFunctionReturnValueView, AnnotatedStateView, EventPage,
    ExecuteTransactionResponseView, FunctionCallView, StateView, StrView, StructTagView,
    TransactionView,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use moveos_types::access_path::AccessPath;
use moveos_types::event_filter::EventFilter;
use rooch_types::H256;

#[rpc(server, client)]
pub trait RoochAPI {
    /// Send the signed transaction in bcs hex format
    /// This method does not block waiting for the transaction to be executed.
    #[method(name = "rooch_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx_bcs_hex: StrView<Vec<u8>>) -> RpcResult<H256>;

    /// Send the signed transaction in bcs hex format
    /// This method blocks waiting for the transaction to be executed.
    #[method(name = "rooch_executeRawTransaction")]
    async fn execute_raw_transaction(
        &self,
        tx_bcs_hex: StrView<Vec<u8>>,
    ) -> RpcResult<ExecuteTransactionResponseView>;

    /// Execute a read-only function call
    /// The function do not change the state of Application
    #[method(name = "rooch_executeViewFunction")]
    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<Vec<AnnotatedFunctionReturnValueView>>;

    /// Get the states by access_path
    #[method(name = "rooch_getStates")]
    async fn get_states(&self, access_path: AccessPath) -> RpcResult<Vec<Option<StateView>>>;

    /// Get the annotated states by access_path
    /// The annotated states include the decoded move value of the state
    #[method(name = "rooch_getAnnotatedStates")]
    async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> RpcResult<Vec<Option<AnnotatedStateView>>>;

    /// Get the events by event handle id
    #[method(name = "rooch_getEventsByEventHandle")]
    async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<u64>,
        limit: Option<u64>,
    ) -> RpcResult<EventPage>;

    /// Get the events by event filter
    #[method(name = "rooch_getEvents")]
    async fn get_events(&self, filter: EventFilter) -> RpcResult<Vec<Option<AnnotatedEventView>>>;

    #[method(name = "rooch_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<Option<TransactionView>>;

    #[method(name = "rooch_getTransactionByIndex")]
    async fn get_transaction_by_index(
        &self,
        start: u64,
        limit: u64,
    ) -> RpcResult<Vec<TransactionView>>;
}
