// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{Bytes, TransactionRequest};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_types::H256;

// Define a rpc server api
#[rpc(server, client)]
pub trait EthAPI {
    /// Sends transaction; will block waiting for signer to return the
    /// transaction hash.
    #[method(name = "eth_sendTransaction")]
    async fn send_transaction(&self, _request: TransactionRequest) -> RpcResult<H256> {
        // the `eth_sendTransaction` method is not supported by this server
        // because it requires a signer to be available.
        // Please use the `eth_sendRawTransaction` method instead.
        //TODO find a suitable error code
        Err(jsonrpsee::core::Error::Custom("eth_sendTransaction is not supported by this server. Please use eth_sendRawTransaction instead.".to_string()))
    }

    /// Sends signed transaction, returning its hash.
    #[method(name = "eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<H256>;
}
