// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::access_path::AccessPath;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::h256::H256;
use moveos_types::moveos_std::account::Account;
use moveos_types::moveos_std::event::{AnnotatedEvent, Event, EventID};
use moveos_types::state::{AnnotatedState, KeyState, MoveStructType, State};
use moveos_types::state_resolver::{AnnotatedStateKV, StateKV};
use moveos_types::transaction::{FunctionCall, TransactionExecutionInfo};
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::proxy::IndexerProxy;
use rooch_pipeline_processor::proxy::PipelineProcessorProxy;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::address::MultiChainAddress;
use rooch_types::indexer::event_filter::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::indexer::state::{
    FieldStateFilter, IndexerFieldState, IndexerObjectState, IndexerStateID, IndexerTableChangeSet,
    ObjectStateFilter, StateSyncFilter,
};
use rooch_types::indexer::transaction_filter::TransactionFilter;
use rooch_types::sequencer::SequencerOrder;
use rooch_types::transaction::{
    ExecuteTransactionResponse, LedgerTransaction, RoochTransaction, TransactionWithInfo,
};

/// RpcService is the implementation of the RPC service.
/// It is the glue between the RPC server(EthAPIServer,RoochApiServer) and the rooch's actors.
/// The RpcService encapsulates the logic of the functions, and the RPC server handle the response format.
#[derive(Clone)]
pub struct RpcService {
    pub(crate) executor: ExecutorProxy,
    pub(crate) sequencer: SequencerProxy,
    pub(crate) indexer: IndexerProxy,
    pub(crate) pipeline_processor: PipelineProcessorProxy,
}

impl RpcService {
    pub fn new(
        executor: ExecutorProxy,
        sequencer: SequencerProxy,
        indexer: IndexerProxy,
        pipeline_processor: PipelineProcessorProxy,
    ) -> Self {
        Self {
            executor,
            sequencer,
            indexer,
            pipeline_processor,
        }
    }
}

impl RpcService {
    pub async fn get_chain_id(&self) -> Result<u64> {
        Ok(self.executor.chain_id().await?.id)
    }

    pub async fn get_bitcoin_network(&self) -> Result<u8> {
        Ok(self.executor.bitcoin_network().await?.network)
    }

    pub async fn quene_tx(&self, tx: RoochTransaction) -> Result<()> {
        //TODO implement quene tx and do not wait to execute
        let _ = self.execute_tx(tx).await?;
        Ok(())
    }

    pub async fn execute_tx(&self, tx: RoochTransaction) -> Result<ExecuteTransactionResponse> {
        self.pipeline_processor.execute_l2_tx(tx).await
    }

    pub async fn execute_view_function(
        &self,
        function_call: FunctionCall,
    ) -> Result<AnnotatedFunctionResult> {
        let resp = self.executor.execute_view_function(function_call).await?;
        Ok(resp)
    }

    pub async fn resolve_address(&self, mca: MultiChainAddress) -> Result<AccountAddress> {
        self.executor.resolve_address(mca).await
    }

    pub async fn get_states(&self, access_path: AccessPath) -> Result<Vec<Option<State>>> {
        self.executor.get_states(access_path).await
    }

    pub async fn exists_account(&self, address: AccountAddress) -> Result<bool> {
        let mut resp = self
            .get_states(AccessPath::resource(address, Account::struct_tag()))
            .await?;
        Ok(resp.pop().flatten().is_some())
    }

    pub async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> Result<Vec<Option<AnnotatedState>>> {
        self.executor.get_annotated_states(access_path).await
    }

    pub async fn list_states(
        &self,
        access_path: AccessPath,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        self.executor.list_states(access_path, cursor, limit).await
    }

    pub async fn list_annotated_states(
        &self,
        access_path: AccessPath,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<AnnotatedStateKV>> {
        self.executor
            .list_annotated_states(access_path, cursor, limit)
            .await
    }

    pub async fn get_annotated_events_by_event_handle(
        &self,
        event_handle_type: StructTag,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<AnnotatedEvent>> {
        let resp = self
            .executor
            .get_annotated_events_by_event_handle(
                event_handle_type,
                cursor,
                limit,
                descending_order,
            )
            .await?;
        Ok(resp)
    }

    pub async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTag,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<Event>> {
        let resp = self
            .executor
            .get_events_by_event_handle(event_handle_type, cursor, limit, descending_order)
            .await?;
        Ok(resp)
    }

    pub async fn get_events_by_event_ids(
        &self,
        event_ids: Vec<EventID>,
    ) -> Result<Vec<Option<AnnotatedEvent>>> {
        let resp = self.executor.get_events_by_event_ids(event_ids).await?;
        Ok(resp)
    }

    pub async fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>> {
        let resp = self.sequencer.get_transaction_by_hash(hash).await?;
        Ok(resp)
    }

    pub async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<LedgerTransaction>>> {
        let resp = self.sequencer.get_transactions_by_hash(tx_hashes).await?;
        Ok(resp)
    }

    pub async fn get_tx_hashs(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>> {
        let resp = self.sequencer.get_tx_hashs(tx_orders).await?;
        Ok(resp)
    }

    pub async fn get_transaction_execution_infos_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        let resp = self
            .executor
            .get_transaction_execution_infos_by_hash(tx_hashes)
            .await?;
        Ok(resp)
    }

    pub async fn get_sequencer_order(&self) -> Result<Option<SequencerOrder>> {
        let resp = self.sequencer.get_sequencer_order().await?;
        Ok(resp)
    }

    pub async fn get_annotated_states_by_state(
        &self,
        states: Vec<State>,
    ) -> Result<Vec<AnnotatedState>> {
        let resp = self.executor.get_annotated_states_by_state(states).await?;
        Ok(resp)
    }

    pub async fn query_transactions(
        &self,
        filter: TransactionFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<u64>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<TransactionWithInfo>> {
        let resp = self
            .indexer
            .query_transactions(filter, cursor, limit, descending_order)
            .await?;
        Ok(resp)
    }

    pub async fn query_events(
        &self,
        filter: EventFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerEventID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerEvent>> {
        // ) -> Result<Vec<AnnotatedEvent>> {

        let resp = self
            .indexer
            .query_events(filter, cursor, limit, descending_order)
            .await?;
        Ok(resp)
    }

    pub async fn query_object_states(
        &self,
        filter: ObjectStateFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerObjectState>> {
        let resp = self
            .indexer
            .query_object_states(filter, cursor, limit, descending_order)
            .await?;
        Ok(resp)
    }

    pub async fn query_field_states(
        &self,
        filter: FieldStateFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerFieldState>> {
        let resp = self
            .indexer
            .query_field_states(filter, cursor, limit, descending_order)
            .await?;
        Ok(resp)
    }

    pub async fn sync_states(
        &self,
        filter: Option<StateSyncFilter>,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerTableChangeSet>> {
        let resp = self
            .indexer
            .sync_states(filter, cursor, limit, descending_order)
            .await?;
        Ok(resp)
    }
}
