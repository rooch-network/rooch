// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use jsonrpsee::http_client::HttpClient;
use moveos_types::h256::H256;
use moveos_types::{
    access_path::AccessPath,
    state::{MoveStructType, State},
    transaction::FunctionCall,
};
use rooch_rpc_api::api::rooch_api::RoochAPIClient;
use rooch_rpc_api::jsonrpc_types::TransactionWithInfoPageView;
use rooch_rpc_api::jsonrpc_types::{
    account_view::BalanceInfoView, transaction_view::TransactionWithInfoView,
};
use rooch_rpc_api::jsonrpc_types::{
    AccessPathView, AccountAddressView, AnnotatedFunctionResultView, AnnotatedStatesPageView,
    BalanceInfoPageView, EventPageView, StatesPageView, StrView, StructTagView,
};
use rooch_rpc_api::jsonrpc_types::{AnnotatedStateView, ExecuteTransactionResponseView, StateView};
use rooch_types::{account::Account, address::RoochAddress, transaction::rooch::RoochTransaction};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RoochRpcClient {
    http: Arc<HttpClient>,
}

// TODO: call args are uniformly defined in jsonrpc types?
// example execute_view_function get_events_by_event_handle

impl RoochRpcClient {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    pub async fn get_chain_id(&self) -> Result<u64> {
        Ok(self.http.get_chain_id().await?.0)
    }

    pub async fn execute_tx(&self, tx: RoochTransaction) -> Result<ExecuteTransactionResponseView> {
        let tx_payload = bcs::to_bytes(&tx)?;
        self.http
            .execute_raw_transaction(tx_payload.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn execute_view_function(
        &self,
        function_call: FunctionCall,
    ) -> Result<AnnotatedFunctionResultView> {
        self.http
            .execute_view_function(function_call.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn get_states(&self, access_path: AccessPath) -> Result<Vec<Option<StateView>>> {
        Ok(self.http.get_states(access_path.into()).await?)
    }

    pub async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> Result<Vec<Option<AnnotatedStateView>>> {
        Ok(self.http.get_annotated_states(access_path.into()).await?)
    }

    // pub async fn get_transactions_by_hash(&self, hash: H256) -> Result<Option<TransactionView>> {
    //     Ok(self.http.get_transaction_by_hash(hash.into()).await?)
    // }

    pub async fn get_transactions_by_order(
        &self,
        cursor: Option<u128>,
        limit: Option<u64>,
    ) -> Result<TransactionWithInfoPageView> {
        Ok(self.http.get_transactions_by_order(cursor, limit).await?)
    }

    pub async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionWithInfoView>>> {
        Ok(self
            .http
            .get_transactions_by_hash(tx_hashes.iter().map(|hash| (*hash).into()).collect())
            .await?)
    }

    pub async fn get_sequence_number(&self, sender: RoochAddress) -> Result<u64> {
        Ok(self
            .get_states(AccessPath::resource(sender.into(), Account::struct_tag()))
            .await?
            .pop()
            .flatten()
            .map(|state_view| {
                let state = State::from(state_view);
                state.as_move_state::<Account>()
            })
            .transpose()?
            .map_or(0, |account| account.sequence_number))
    }

    pub async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<u64>,
        limit: Option<u64>,
    ) -> Result<EventPageView> {
        let s = self
            .http
            .get_events_by_event_handle(event_handle_type, cursor, limit)
            .await?;
        Ok(s)
    }

    pub async fn list_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> Result<StatesPageView> {
        Ok(self.http.list_states(access_path, cursor, limit).await?)
    }

    pub async fn list_annotated_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> Result<AnnotatedStatesPageView> {
        Ok(self
            .http
            .list_annotated_states(access_path, cursor, limit)
            .await?)
    }

    pub async fn get_balance(
        &self,
        account_addr: AccountAddressView,
        coin_type: StructTagView,
    ) -> Result<BalanceInfoView> {
        Ok(self.http.get_balance(account_addr, coin_type).await?)
    }

    pub async fn get_balances(
        &self,
        account_addr: AccountAddressView,
        cursor: Option<StrView<Vec<u8>>>,
        limit: Option<usize>,
    ) -> Result<BalanceInfoPageView> {
        Ok(self.http.get_balances(account_addr, cursor, limit).await?)
    }
}
