// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    GetTxExecutionInfosByHashMessage, ListAnnotatedStatesMessage, ListStatesMessage,
};
use crate::actor::{
    executor::ExecutorActor,
    messages::{
        AnnotatedStatesMessage, ExecuteViewFunctionMessage, GetEventsByEventHandleMessage,
        GetEventsMessage, ResolveMessage, StatesMessage, ValidateTransactionMessage,
    },
};
use anyhow::Result;
use coerce::actor::ActorRef;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::h256::H256;
use moveos_types::transaction::FunctionCall;
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::transaction::TransactionOutput;
use moveos_types::{access_path::AccessPath, transaction::VerifiedMoveOSTransaction};
use moveos_types::{
    event_filter::EventFilter,
    moveos_std::event::AnnotatedEvent,
    state::{AnnotatedState, State},
};
use rooch_types::address::MultiChainAddress;
use rooch_types::transaction::AbstractTransaction;

#[derive(Clone)]
pub struct ExecutorProxy {
    pub actor: ActorRef<ExecutorActor>,
}

impl ExecutorProxy {
    pub fn new(actor: ActorRef<ExecutorActor>) -> Self {
        Self { actor }
    }

    pub async fn validate_transaction<T>(&self, tx: T) -> Result<VerifiedMoveOSTransaction>
    where
        T: 'static + AbstractTransaction + Send + Sync,
    {
        self.actor.send(ValidateTransactionMessage { tx }).await?
    }

    //TODO ensure the execute result
    pub async fn execute_transaction(
        &self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<(TransactionOutput, TransactionExecutionInfo)> {
        let result = self
            .actor
            .send(crate::actor::messages::ExecuteTransactionMessage { tx })
            .await??;
        Ok((result.output, result.transaction_info))
    }

    pub async fn execute_view_function(
        &self,
        call: FunctionCall,
    ) -> Result<AnnotatedFunctionResult> {
        self.actor.send(ExecuteViewFunctionMessage { call }).await?
    }

    pub async fn get_states(&self, access_path: AccessPath) -> Result<Vec<Option<State>>> {
        self.actor.send(StatesMessage { access_path }).await?
    }

    pub async fn resolve_address(&self, mca: MultiChainAddress) -> Result<AccountAddress> {
        self.actor.send(ResolveMessage { address: mca }).await?
    }

    pub async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> Result<Vec<Option<AnnotatedState>>> {
        self.actor
            .send(AnnotatedStatesMessage { access_path })
            .await?
    }

    pub async fn list_states(
        &self,
        access_path: AccessPath,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, State)>> {
        self.actor
            .send(ListStatesMessage {
                access_path,
                cursor,
                limit,
            })
            .await?
    }

    pub async fn list_annotated_states(
        &self,
        access_path: AccessPath,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, AnnotatedState)>> {
        self.actor
            .send(ListAnnotatedStatesMessage {
                access_path,
                cursor,
                limit,
            })
            .await?
    }

    pub async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTag,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<AnnotatedEvent>> {
        self.actor
            .send(GetEventsByEventHandleMessage {
                event_handle_type,
                cursor,
                limit,
            })
            .await?
    }

    pub async fn get_events(&self, filter: EventFilter) -> Result<Vec<AnnotatedEvent>> {
        self.actor.send(GetEventsMessage { filter }).await?
    }

    pub async fn get_transaction_execution_infos_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        self.actor
            .send(GetTxExecutionInfosByHashMessage { tx_hashes })
            .await?
    }
}
