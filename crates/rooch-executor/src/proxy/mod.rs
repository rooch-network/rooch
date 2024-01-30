// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    GetAnnotatedStatesByStateMessage, GetEventsByEventHandleMessage, GetEventsByEventIDsMessage,
    GetTxExecutionInfosByHashMessage, ListAnnotatedStatesMessage, ListStatesMessage,
    RefreshStateMessage,
};
use crate::actor::reader_executor::ReaderExecutorActor;
use crate::actor::{
    executor::ExecutorActor,
    messages::{
        AnnotatedStatesMessage, ExecuteViewFunctionMessage, GetAnnotatedEventsByEventHandleMessage,
        ResolveMessage, StatesMessage, ValidateTransactionMessage,
    },
};
use anyhow::Result;
use coerce::actor::ActorRef;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::function_return_value::{AnnotatedFunctionResult, FunctionResult};
use moveos_types::h256::H256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::event::{Event, EventID};
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::KeyState;
use moveos_types::state_resolver::{AnnotatedStateKV, StateKV};
use moveos_types::transaction::FunctionCall;
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::transaction::TransactionOutput;
use moveos_types::{access_path::AccessPath, transaction::VerifiedMoveOSTransaction};
use moveos_types::{
    moveos_std::event::AnnotatedEvent,
    state::{AnnotatedState, State},
};
use rooch_types::address::MultiChainAddress;
use rooch_types::transaction::AbstractTransaction;
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct ExecutorProxy {
    pub actor: ActorRef<ExecutorActor>,
    pub reader_actor: ActorRef<ReaderExecutorActor>,
}

impl ExecutorProxy {
    pub fn new(
        actor: ActorRef<ExecutorActor>,
        reader_actor: ActorRef<ReaderExecutorActor>,
    ) -> Self {
        Self {
            actor,
            reader_actor,
        }
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
        self.reader_actor
            .send(ExecuteViewFunctionMessage { call })
            .await?
    }

    pub async fn get_states(&self, access_path: AccessPath) -> Result<Vec<Option<State>>> {
        self.reader_actor
            .send(StatesMessage { access_path })
            .await?
    }

    pub async fn resolve_address(&self, mca: MultiChainAddress) -> Result<AccountAddress> {
        self.actor.send(ResolveMessage { address: mca }).await?
    }

    pub async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> Result<Vec<Option<AnnotatedState>>> {
        self.reader_actor
            .send(AnnotatedStatesMessage { access_path })
            .await?
    }

    pub async fn list_states(
        &self,
        access_path: AccessPath,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        self.reader_actor
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
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<AnnotatedStateKV>> {
        self.reader_actor
            .send(ListAnnotatedStatesMessage {
                access_path,
                cursor,
                limit,
            })
            .await?
    }

    pub async fn get_annotated_events_by_event_handle(
        &self,
        event_handle_type: StructTag,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<AnnotatedEvent>> {
        self.reader_actor
            .send(GetAnnotatedEventsByEventHandleMessage {
                event_handle_type,
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
    ) -> Result<Vec<Event>> {
        self.reader_actor
            .send(GetEventsByEventHandleMessage {
                event_handle_type,
                cursor,
                limit,
            })
            .await?
    }

    pub async fn get_events_by_event_ids(
        &self,
        event_ids: Vec<EventID>,
    ) -> Result<Vec<Option<AnnotatedEvent>>> {
        self.reader_actor
            .send(GetEventsByEventIDsMessage { event_ids })
            .await?
    }

    pub async fn get_transaction_execution_infos_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        self.reader_actor
            .send(GetTxExecutionInfosByHashMessage { tx_hashes })
            .await?
    }

    pub async fn get_annotated_states_by_state(
        &self,
        states: Vec<State>,
    ) -> Result<Vec<AnnotatedState>> {
        self.reader_actor
            .send(GetAnnotatedStatesByStateMessage { states })
            .await?
    }

    pub async fn refresh_state(&self, new_state_root: H256, is_upgrade: bool) -> Result<()> {
        self.reader_actor
            .send(RefreshStateMessage {
                new_state_root,
                is_upgrade,
            })
            .await?
    }
}

impl MoveFunctionCaller for ExecutorProxy {
    fn call_function(
        &self,
        _ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        let executor = self.clone();
        let function_result = tokio::task::block_in_place(|| {
            Handle::current()
                .block_on(async move { executor.execute_view_function(function_call).await })
        })?;
        function_result.try_into()
    }
}
