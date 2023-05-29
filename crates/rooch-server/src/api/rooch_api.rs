// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AnnotatedMoveStructView, AnnotatedObjectView, EventView, FunctionCallView, StrView,
    StructTagView,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use move_core_types::account_address::AccountAddress;
use moveos::moveos::TransactionOutput;
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
    ) -> RpcResult<TransactionOutput>;

    /// Execute a read-only function call
    /// The function do not change the state of Application
    #[method(name = "rooch_executeViewFunction")]
    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<Vec<serde_json::Value>>;

    /// Get the resource of an account by address and type
    #[method(name = "rooch_getResource")]
    async fn get_resource(
        &self,
        address: AccountAddress,
        resource_type: StructTagView,
    ) -> RpcResult<Option<AnnotatedMoveStructView>>;

    #[method(name = "rooch_getObject")]
    async fn get_object(&self, object_id: ObjectID) -> RpcResult<Option<AnnotatedObjectView>>;

    /// Get the events by tx_hash
    #[method(name = "rooch_getEventsByTxHash")]
    async fn get_events_by_tx_hash(&self, tx_hash: H256) -> RpcResult<Option<Vec<EventView>>>;

    /// Get the events by event filter
    #[method(name = "rooch_getEvents")]
    async fn get_events(&self, filter: EventFilter) -> RpcResult<Option<Vec<EventView>>>;
}
