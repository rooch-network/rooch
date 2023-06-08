// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AnnotatedFunctionReturnValueView, AnnotatedMoveStructView, AnnotatedObjectView,
    AnnotatedStateView, EventView, ExecuteTransactionResponse, FunctionCallView, StateView,
    StrView, StructTagView, TransactionView,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use move_core_types::account_address::AccountAddress;
use moveos_types::access_path::AccessPath;
use moveos_types::event_filter::EventFilter;
use moveos_types::object::ObjectID;
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
    ) -> RpcResult<ExecuteTransactionResponse>;

    /// Execute a read-only function call
    /// The function do not change the state of Application
    #[method(name = "rooch_executeViewFunction")]
    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<Vec<AnnotatedFunctionReturnValueView>>;

    /// Get the resource of an account by address and type
    #[method(name = "rooch_getResource")]
    async fn get_resource(
        &self,
        address: AccountAddress,
        resource_type: StructTagView,
    ) -> RpcResult<Option<AnnotatedMoveStructView>>;

    #[method(name = "rooch_getObject")]
    async fn get_object(&self, object_id: ObjectID) -> RpcResult<Option<AnnotatedObjectView>>;

    //TODO should we merge the `get_resource` and `get_object` to `get_state`?
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
        event_handle_id: ObjectID,
    ) -> RpcResult<Option<Vec<EventView>>>;

    /// Get the events by event filter
    #[method(name = "rooch_getEvents")]
    async fn get_events(&self, filter: EventFilter) -> RpcResult<Option<Vec<EventView>>>;

    #[method(name = "rooch_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<Option<TransactionView>>;

    #[method(name = "rooch_getTransactionByIndex")]
    async fn get_transaction_by_index(
        &self,
        start: u64,
        limit: u64,
    ) -> RpcResult<Vec<TransactionView>>;
}
