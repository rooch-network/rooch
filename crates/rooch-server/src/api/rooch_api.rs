// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::Bytes;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use moveos::moveos::TransactionOutput;
use moveos_types::object::ObjectID;
use rooch_types::H256;

#[rpc(server, client)]
pub trait RoochAPI {
    #[method(name = "echo")]
    async fn echo(&self, msg: String) -> RpcResult<String>;

    /// Send the signed transaction in bcs hex format
    /// This method does not block waiting for the transaction to be executed.
    #[method(name = "rooch_sendRawTransaction")]
    async fn send_raw_transaction(&self, payload: Bytes) -> RpcResult<H256>;

    /// Send the signed transaction in bcs hex format
    /// This method blocks waiting for the transaction to be executed.
    #[method(name = "rooch_executeRawTransaction")]
    async fn execute_raw_transaction(&self, payload: Bytes) -> RpcResult<TransactionOutput>;

    #[method(name = "view")]
    async fn view(&self, payload: Vec<u8>) -> RpcResult<Vec<serde_json::Value>>;

    #[method(name = "resource")]
    async fn resource(
        &self,
        address: AccountAddress,
        module: ModuleId,
        resource: Identifier,
        type_args: Vec<TypeTag>,
    ) -> RpcResult<Option<String>>;

    #[method(name = "object")]
    async fn object(&self, object_id: ObjectID) -> RpcResult<Option<String>>;
}
