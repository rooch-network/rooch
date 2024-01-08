// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::access_path::AccessPath;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::{AnnotatedEvent, Event, EventID};
use moveos_types::state::{AnnotatedState, State};
use moveos_types::transaction::FunctionCall;
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::transaction::TransactionOutput;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use rooch_types::address::MultiChainAddress;
use rooch_types::transaction::AbstractTransaction;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ValidateTransactionMessage<T> {
    pub tx: T,
}

impl<T> Message for ValidateTransactionMessage<T>
where
    T: 'static + AbstractTransaction + Send + Sync,
{
    type Result = Result<VerifiedMoveOSTransaction>;
}

#[derive(Debug)]
pub struct ExecuteTransactionMessage {
    pub tx: VerifiedMoveOSTransaction,
}

#[derive(Debug)]
pub struct ExecuteTransactionResult {
    pub output: TransactionOutput,
    pub transaction_info: TransactionExecutionInfo,
}

impl Message for ExecuteTransactionMessage {
    type Result = Result<ExecuteTransactionResult>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteViewFunctionMessage {
    pub call: FunctionCall,
}

impl Message for ExecuteViewFunctionMessage {
    type Result = Result<AnnotatedFunctionResult, anyhow::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatesMessage {
    pub access_path: AccessPath,
}

impl Message for StatesMessage {
    type Result = Result<Vec<Option<State>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveMessage {
    pub address: MultiChainAddress,
}

impl Message for ResolveMessage {
    type Result = Result<AccountAddress>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotatedStatesMessage {
    pub access_path: AccessPath,
}

impl Message for AnnotatedStatesMessage {
    type Result = Result<Vec<Option<AnnotatedState>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListStatesMessage {
    pub access_path: AccessPath,
    pub cursor: Option<Vec<u8>>,
    pub limit: usize,
}

impl Message for ListStatesMessage {
    type Result = Result<Vec<(Vec<u8>, State)>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAnnotatedStatesMessage {
    pub access_path: AccessPath,
    pub cursor: Option<Vec<u8>>,
    pub limit: usize,
}

impl Message for ListAnnotatedStatesMessage {
    type Result = Result<Vec<(Vec<u8>, AnnotatedState)>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAnnotatedEventsByEventHandleMessage {
    pub event_handle_type: StructTag,
    pub cursor: Option<u64>,
    pub limit: u64,
}

impl Message for GetAnnotatedEventsByEventHandleMessage {
    type Result = Result<Vec<AnnotatedEvent>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsByEventHandleMessage {
    pub event_handle_type: StructTag,
    pub cursor: Option<u64>,
    pub limit: u64,
}

impl Message for GetEventsByEventHandleMessage {
    type Result = Result<Vec<Event>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsByEventIDsMessage {
    pub event_ids: Vec<EventID>,
}

impl Message for GetEventsByEventIDsMessage {
    type Result = Result<Vec<Option<AnnotatedEvent>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxExecutionInfosByHashMessage {
    pub tx_hashes: Vec<H256>,
}

impl Message for GetTxExecutionInfosByHashMessage {
    type Result = Result<Vec<Option<TransactionExecutionInfo>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAnnotatedStatesByStateMessage {
    pub states: Vec<State>,
}

impl Message for GetAnnotatedStatesByStateMessage {
    type Result = Result<Vec<AnnotatedState>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshStateMessage {
    pub new_state_root: H256,
    pub is_upgrade: bool,
}

impl Message for RefreshStateMessage {
    type Result = Result<()>;
}
