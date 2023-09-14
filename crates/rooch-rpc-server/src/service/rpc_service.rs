// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::access_path::AccessPath;
use moveos_types::event::AnnotatedMoveOSEvent;
use moveos_types::event_filter::EventFilter;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::state::{AnnotatedState, State};
use moveos_types::transaction::{FunctionCall, TransactionExecutionInfo};
use rooch_executor::proxy::ExecutorProxy;
use rooch_proposer::proxy::ProposerProxy;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponse;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::address::{MultiChainAddress, RoochAddress};
use rooch_types::transaction::{TransactionSequenceInfo, TransactionSequenceMapping};
use rooch_types::{transaction::TypedTransaction, H256};

/// RpcService is the implementation of the RPC service.
/// It is the glue between the RPC server(EthAPIServer,RoochApiServer) and the rooch's actors.
/// The RpcService encapsulates the logic of the functions, and the RPC server handle the response format.
#[derive(Clone)]
pub struct RpcService {
    executor: ExecutorProxy,
    sequencer: SequencerProxy,
    proposer: ProposerProxy,
}

impl RpcService {
    pub fn new(
        executor: ExecutorProxy,
        sequencer: SequencerProxy,
        proposer: ProposerProxy,
    ) -> Self {
        Self {
            executor,
            sequencer,
            proposer,
        }
    }
}

impl RpcService {
    pub async fn quene_tx(&self, tx: TypedTransaction) -> Result<()> {
        //TODO implement quene tx and do not wait to execute
        let _ = self.execute_tx(tx).await?;
        Ok(())
    }

    pub async fn execute_tx(&self, tx: TypedTransaction) -> Result<ExecuteTransactionResponse> {
        //First, validate the transactin
        let moveos_tx = self.executor.validate_transaction(tx.clone()).await?;
        let sequence_info = self.sequencer.sequence_transaction(tx.clone()).await?;
        // Then execute
        let (output, execution_info) = self.executor.execute_transaction(moveos_tx).await?;
        self.proposer
            .propose_transaction(tx, execution_info.clone(), sequence_info.clone())
            .await?;

        Ok(ExecuteTransactionResponse {
            sequence_info,
            execution_info,
            output,
        })
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

    pub async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> Result<Vec<Option<AnnotatedState>>> {
        self.executor.get_annotated_states(access_path).await
    }

    pub async fn list_states(
        &self,
        access_path: AccessPath,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<Option<(Vec<u8>, State)>>> {
        self.executor.list_states(access_path, cursor, limit).await
    }

    pub async fn list_annotated_states(
        &self,
        access_path: AccessPath,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<Option<(Vec<u8>, AnnotatedState)>>> {
        self.executor
            .list_annotated_states(access_path, cursor, limit)
            .await
    }

    /// Sign a message with the private key of the given address.
    pub async fn sign(&self, _address: RoochAddress, _message: Vec<u8>) -> Result<Vec<u8>> {
        bail!("Not implemented")
        //TODO implement sign
        //Call WalletActor to sign?
        //How to unlock the wallet?
        //Define the sign message format for rooch, and does it need to be compatible with Ethereum?
    }

    pub async fn accounts(&self) -> Result<Vec<RoochAddress>> {
        bail!("Not implemented")
    }

    pub async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTag,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Option<AnnotatedMoveOSEvent>>> {
        let resp = self
            .executor
            .get_events_by_event_handle(event_handle_type, cursor, limit)
            .await?;
        Ok(resp)
    }

    pub async fn get_events(
        &self,
        filter: EventFilter,
    ) -> Result<Vec<Option<AnnotatedMoveOSEvent>>> {
        let resp = self.executor.get_events(filter).await?;
        Ok(resp)
    }

    pub async fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<TypedTransaction>> {
        let resp = self.sequencer.get_transaction_by_hash(hash).await?;
        Ok(resp)
    }

    pub async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TypedTransaction>>> {
        let resp = self.sequencer.get_transactions_by_hash(tx_hashes).await?;
        Ok(resp)
    }

    pub async fn get_transaction_sequence_infos(
        &self,
        orders: Vec<u128>,
    ) -> Result<Vec<Option<TransactionSequenceInfo>>> {
        let resp = self
            .sequencer
            .get_transaction_sequence_infos(orders)
            .await?;
        Ok(resp)
    }

    pub async fn get_tx_sequence_mapping_by_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceMapping>> {
        let resp = self
            .sequencer
            .get_transaction_sequence_mapping_by_order(cursor, limit)
            .await?;
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
}
