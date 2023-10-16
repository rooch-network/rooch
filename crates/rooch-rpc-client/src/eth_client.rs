// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use ethers::types::{H160, H256, U256};
use jsonrpsee::http_client::HttpClient;
use rooch_rpc_api::api::eth_api::EthAPIClient;
use rooch_rpc_api::jsonrpc_types::{H256View, StrView};
use rooch_rpc_api::{
    api::eth_api::TransactionType,
    jsonrpc_types::{
        eth::{
            ethereum_types::block::{Block, BlockNumber},
            CallRequest, EthFeeHistory, Transaction, TransactionReceipt,
        },
        BytesView,
    },
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct EthRpcClient {
    http: Arc<HttpClient>,
}

impl EthRpcClient {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    pub async fn chain_id(&self) -> Result<String> {
        Ok(self.http.chain_id().await?)
    }

    pub async fn get_block_number(&self) -> Result<String> {
        Ok(self.http.get_block_number().await?)
    }

    pub async fn get_block_by_number(
        &self,
        num: BlockNumber,
        include_txs: bool,
    ) -> Result<Block<TransactionType>> {
        Ok(self
            .http
            .get_block_by_number(num.into(), include_txs)
            .await?)
    }

    pub async fn get_balance(
        &self,
        address: H160,
        num: Option<BlockNumber>,
    ) -> Result<StrView<U256>> {
        let response = self
            .http
            .get_balance(address.into(), num.map(StrView))
            .await?;
        Result::Ok(response)
    }

    pub async fn estimate_gas(
        &self,
        request: CallRequest,
        num: Option<BlockNumber>,
    ) -> Result<StrView<U256>> {
        let response = self.http.estimate_gas(request, num.map(StrView)).await?;
        Result::Ok(response)
    }

    pub async fn fee_history(
        &self,
        block_count: U256,
        newest_block: BlockNumber,
        reward_percentiles: Option<Vec<f64>>,
    ) -> Result<EthFeeHistory> {
        Ok(self
            .http
            .fee_history(block_count.into(), newest_block.into(), reward_percentiles)
            .await?)
    }

    pub async fn gas_price(&self) -> Result<StrView<U256>> {
        let response = self.http.gas_price().await?;
        Result::Ok(response)
    }

    pub async fn transaction_count(
        &self,
        address: H160,
        num: Option<BlockNumber>,
    ) -> Result<StrView<U256>> {
        let response = self
            .http
            .transaction_count(address.into(), num.map(StrView))
            .await?;
        Result::Ok(response)
    }

    pub async fn send_raw_transaction(&self, bytes: BytesView) -> Result<H256View> {
        let response = self.http.send_raw_transaction(bytes).await?;
        Result::Ok(response)
    }

    pub async fn transaction_receipt(&self, hash: H256) -> Result<Option<TransactionReceipt>> {
        Ok(self.http.transaction_receipt(hash.into()).await?)
    }

    pub async fn transaction_by_hash(&self, hash: H256) -> Result<Option<Transaction>> {
        Ok(self.http.transaction_by_hash(hash.into()).await?)
    }

    pub async fn block_by_hash(
        &self,
        hash: H256,
        include_txs: bool,
    ) -> Result<Block<TransactionType>> {
        Ok(self.http.block_by_hash(hash.into(), include_txs).await?)
    }
}
