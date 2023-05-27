// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{Bytes, TransactionRequest, BlockNumber, Block, TxHash, Transaction};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_types::H256;
use std::string::String;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum TransactionType {
    Full(Transaction),
    Hash(TxHash),
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::Hash(H256::zero())
    }
}

// Define a rpc server api
#[rpc(server, client)]
pub trait EthAPI {
    /// Returns the chain ID of the current network.
    #[method(name = "eth_chainId")]
    async fn get_chain_id(&self) -> RpcResult<String>;

    /// Returns the number of most recent block.
    #[method(name = "eth_blockNumber")]
    async fn get_block_number(&self) -> RpcResult<String>;

    /// Returns information about a block by number.
    #[method(name = "eth_getBlockByNumber")]
    async fn get_block_by_number(&self, num: BlockNumber, include_txs: bool) -> RpcResult<Block<TransactionType>>;

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
