// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    eth::{
        ethereum_types::block::{Block, BlockNumber},
        transaction::{Transaction, TransactionReceipt, TransactionRequest},
        CallRequest, EthFeeHistory,
    },
    BytesView, H160View, H256View, StrView,
};
use ethers::types::{H256, U256};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_open_rpc_macros::open_rpc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::string::String;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug, JsonSchema)]
pub enum TransactionType {
    Full(Transaction),
    Hash(H256View),
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::Hash(H256View::from(H256::zero()))
    }
}

#[open_rpc(namespace = "eth")]
#[rpc(server, client, namespace = "eth")]
#[async_trait]
pub trait EthAPI {
    /// Returns the chain ID of the current network.
    #[method(name = "chainId")]
    async fn chain_id(&self) -> RpcResult<String>;

    /// Returns the number of most recent block.
    #[method(name = "blockNumber")]
    async fn get_block_number(&self) -> RpcResult<String>;

    /// Returns information about a block by number.
    #[method(name = "getBlockByNumber")]
    async fn get_block_by_number(
        &self,
        num: StrView<BlockNumber>,
        include_txs: bool,
    ) -> RpcResult<Block<TransactionType>>;

    /// Returns the balance of the account of given address.
    #[method(name = "getBalance")]
    async fn get_balance(
        &self,
        address: H160View,
        num: Option<StrView<BlockNumber>>,
    ) -> RpcResult<StrView<U256>>;

    /// Generates and returns an estimate of how much gas is necessary to allow the transaction to complete.
    #[method(name = "estimateGas")]
    async fn estimate_gas(
        &self,
        request: CallRequest,
        num: Option<StrView<BlockNumber>>,
    ) -> RpcResult<StrView<U256>>;

    /// Transaction fee history
    #[method(name = "feeHistory")]
    async fn fee_history(
        &self,
        block_count: StrView<U256>,
        newest_block: StrView<BlockNumber>,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<EthFeeHistory>;

    /// Returns the current price per gas in wei.
    #[method(name = "gasPrice")]
    async fn gas_price(&self) -> RpcResult<StrView<U256>>;

    /// Returns the number of transactions sent from an address.
    #[method(name = "getTransactionCount")]
    async fn transaction_count(
        &self,
        address: H160View,
        num: Option<StrView<BlockNumber>>,
    ) -> RpcResult<StrView<U256>>;

    /// Sends transaction; will block waiting for signer to return the
    /// transaction hash.
    #[method(name = "sendTransaction")]
    async fn send_transaction(&self, _request: TransactionRequest) -> RpcResult<H256View> {
        // the `eth_sendTransaction` method is not supported by this server
        // because it requires a signer to be available.
        // Please use the `eth_sendRawTransaction` method instead.
        //TODO find a suitable error code
        Err(jsonrpsee::core::Error::Custom("eth_sendTransaction is not supported by this server. Please use eth_sendRawTransaction instead.".to_owned()))
    }

    /// Sends signed transaction, returning its hash.
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, bytes: BytesView) -> RpcResult<H256View>;

    /// Returns transaction receipt by transaction hash.
    #[method(name = "getTransactionReceipt")]
    async fn transaction_receipt(&self, hash: H256View) -> RpcResult<Option<TransactionReceipt>>;

    /// Get transaction by its hash.
    #[method(name = "getTransactionByHash")]
    async fn transaction_by_hash(&self, hash: H256View) -> RpcResult<Option<Transaction>>;

    /// Returns block with given hash.
    #[method(name = "getBlockByHash")]
    async fn block_by_hash(
        &self,
        hash: H256View,
        include_txs: bool,
    ) -> RpcResult<Block<TransactionType>>;
}

#[open_rpc]
#[rpc(server, client)]
#[async_trait]
pub trait EthNetAPI {
    // The `net_version`` is not in the `eth` namespace,
    // So we put it in a new trait.
    // The metamask will call this method for connecting to the network.
    #[method(name = "net_version")]
    async fn net_version(&self) -> RpcResult<String>;
}
