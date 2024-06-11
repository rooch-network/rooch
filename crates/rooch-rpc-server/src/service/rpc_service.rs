// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::language_storage::{ModuleId, StructTag};
use moveos_types::access_path::AccessPath;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::{AnnotatedEvent, Event, EventID};
use moveos_types::state::{AnnotatedState, KeyState, State};
use moveos_types::state_resolver::{AnnotatedStateKV, StateKV};
use moveos_types::transaction::{FunctionCall, TransactionExecutionInfo};
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::proxy::IndexerProxy;
use rooch_pipeline_processor::proxy::PipelineProcessorProxy;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::address::{BitcoinAddress, RoochAddress};
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::indexer::event::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::indexer::state::{IndexerObjectState, IndexerStateID, ObjectStateFilter};
use rooch_types::indexer::transaction::{IndexerTransaction, TransactionFilter};
use rooch_types::transaction::{ExecuteTransactionResponse, LedgerTransaction, RoochTransaction};
use std::collections::HashMap;

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

    pub async fn queue_tx(&self, tx: RoochTransaction) -> Result<()> {
        //TODO implement queue tx and do not wait to execute
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
        let module_id = function_call.function_id.module_id.clone();
        if !self.exists_module(module_id.clone()).await? {
            return Err(anyhow::anyhow!("Module does not exist: {}", module_id));
        }

        let resp = self.executor.execute_view_function(function_call).await?;
        Ok(resp)
    }

    pub async fn get_states(&self, access_path: AccessPath) -> Result<Vec<Option<State>>> {
        self.executor.get_states(access_path).await
    }

    pub async fn exists_module(&self, module_id: ModuleId) -> Result<bool> {
        let mut resp = self.get_states(AccessPath::module(&module_id)).await?;
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

    pub async fn get_sequencer_order(&self) -> Result<u64> {
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
    ) -> Result<Vec<IndexerTransaction>> {
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

    pub async fn get_bitcoin_addresses(
        &self,
        rooch_addresses: Vec<RoochAddress>,
    ) -> Result<HashMap<RoochAddress, Option<BitcoinAddress>>> {
        let mapping_object_id = RoochToBitcoinAddressMapping::object_id();
        let owner_keys = rooch_addresses
            .iter()
            .map(|addr| KeyState::from_address((*addr).into()))
            .collect::<Vec<_>>();

        let access_path = AccessPath::fields(mapping_object_id, owner_keys);
        let address_mapping = self
            .get_states(access_path)
            .await?
            .into_iter()
            .zip(rooch_addresses)
            .map(|(state_opt, owner)| {
                Ok((
                    owner,
                    state_opt
                        .map(|state| state.cast_unchecked::<BitcoinAddress>())
                        .transpose()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        Ok(address_mapping)
    }
}
