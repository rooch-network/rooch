// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{Bytes, TransactionRequest, BlockNumber, Block, TxHash, Transaction, H160, U256};
use crate::jsonrpc_types::eth::CallRequest;
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
    

    /// Returns the network version.
    #[method(name = "net_version")]
    async fn net_version(&self) -> RpcResult<String>;

    /// Returns the chain ID of the current network.
    #[method(name = "eth_chainId")]
    async fn get_chain_id(&self) -> RpcResult<String>;

    /// Returns the number of most recent block.
    #[method(name = "eth_blockNumber")]
    async fn get_block_number(&self) -> RpcResult<String>;

    /// Returns information about a block by number.
    #[method(name = "eth_getBlockByNumber")]
    async fn get_block_by_number(&self, num: BlockNumber, include_txs: bool) -> RpcResult<Block<TransactionType>>;

    /// Returns the balance of the account of given address.
    #[method(name = "eth_getBalance")]
    async fn get_balance(&self, address: H160, num: Option<BlockNumber>) -> RpcResult<U256>;

    /// Generates and returns an estimate of how much gas is necessary to allow the transaction to complete.
    #[method(name = "eth_estimateGas")]
    async fn estimate_gas(&self, request: CallRequest, num: Option<BlockNumber>) -> RpcResult<U256>;

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
