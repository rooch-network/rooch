// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use moveos_types::transaction::FunctionCall;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_rpc_client::Client;
use rooch_types::{address::RoochAddress, transaction::rooch::RoochTransaction};

pub mod actor;

#[async_trait]
pub trait Relayer: Send + Sync {
    fn name(&self) -> &'static str {
        return std::any::type_name::<Self>();
    }

    async fn relay(&mut self) -> Result<Option<FunctionCall>>;
}

#[async_trait]
pub trait TxSubmiter: Send + Sync {
    async fn get_chain_id(&self) -> Result<u64>;
    async fn get_sequence_number(&self, address: RoochAddress) -> Result<u64>;
    async fn submit_tx(&self, tx: RoochTransaction) -> Result<ExecuteTransactionResponseView>;
}

#[async_trait]
impl TxSubmiter for Client {
    async fn get_chain_id(&self) -> Result<u64> {
        self.get_chain_id().await
    }
    async fn get_sequence_number(&self, address: RoochAddress) -> Result<u64> {
        self.get_sequence_number(address).await
    }
    async fn submit_tx(&self, tx: RoochTransaction) -> Result<ExecuteTransactionResponseView> {
        self.execute_tx(tx).await
    }
}
