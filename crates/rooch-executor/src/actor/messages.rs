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
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::state::{AnnotatedState, FieldKey, ObjectState, StateChangeSetExt};
use moveos_types::state_resolver::{AnnotatedStateKV, StateKV};
use moveos_types::state_root_hash::StateRootHash;
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::transaction::TransactionOutput;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use moveos_types::transaction::{FunctionCall, RawTransactionOutput, VMErrorInfo};
use rooch_types::address::MultiChainAddress;
use rooch_types::transaction::{
    L1BlockWithBody, L1Transaction, RoochTransaction, RoochTransactionData,
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ValidateL2TxMessage {
    pub tx: RoochTransaction,
}

impl Message for ValidateL2TxMessage {
    type Result = Result<VerifiedMoveOSTransaction>;
}

#[derive(Debug)]
pub struct ConvertL2TransactionData {
    pub tx_data: RoochTransactionData,
}

impl Message for ConvertL2TransactionData {
    type Result = Result<VerifiedMoveOSTransaction>;
}

#[derive(Debug)]
pub struct ValidateL1BlockMessage {
    pub l1_block: L1BlockWithBody,
}

impl Message for ValidateL1BlockMessage {
    type Result = Result<VerifiedMoveOSTransaction>;
}

#[derive(Debug)]
pub struct ValidateL1TxMessage {
    pub l1_tx: L1Transaction,
}

impl Message for ValidateL1TxMessage {
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

#[derive(Debug)]
pub struct DryRunTransactionMessage {
    pub tx: VerifiedMoveOSTransaction,
}

impl Message for DryRunTransactionMessage {
    type Result = Result<DryRunTransactionResult>;
}

#[derive(Debug)]
pub struct DryRunTransactionResult {
    pub raw_output: RawTransactionOutput,
    pub vm_error_info: Option<VMErrorInfo>,
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
    pub state_root: StateRootHash,
    pub access_path: AccessPath,
}

impl Message for StatesMessage {
    type Result = Result<Vec<Option<ObjectState>>>;
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
    pub cursor: Option<FieldKey>,
    pub limit: usize,
}

impl Message for ListStatesMessage {
    type Result = Result<Vec<StateKV>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAnnotatedStatesMessage {
    pub access_path: AccessPath,
    pub cursor: Option<FieldKey>,
    pub limit: usize,
}

impl Message for ListAnnotatedStatesMessage {
    type Result = Result<Vec<AnnotatedStateKV>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAnnotatedEventsByEventHandleMessage {
    pub event_handle_type: StructTag,
    pub cursor: Option<u64>,
    pub limit: u64,
    pub descending_order: bool,
}

impl Message for GetAnnotatedEventsByEventHandleMessage {
    type Result = Result<Vec<AnnotatedEvent>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsByEventHandleMessage {
    pub event_handle_type: StructTag,
    pub cursor: Option<u64>,
    pub limit: u64,
    pub descending_order: bool,
}

impl Message for GetEventsByEventHandleMessage {
    type Result = Result<Vec<Event>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAnnotatedEventsByEventIDsMessage {
    pub event_ids: Vec<EventID>,
}

impl Message for GetAnnotatedEventsByEventIDsMessage {
    type Result = Result<Vec<Option<AnnotatedEvent>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsByEventIDsMessage {
    pub event_ids: Vec<EventID>,
}

impl Message for GetEventsByEventIDsMessage {
    type Result = Result<Vec<Option<Event>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxExecutionInfosByHashMessage {
    pub tx_hashes: Vec<H256>,
}

impl Message for GetTxExecutionInfosByHashMessage {
    type Result = Result<Vec<Option<TransactionExecutionInfo>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshStateMessage {
    pub root: ObjectMeta,
    pub is_upgrade: bool,
}

impl Message for RefreshStateMessage {
    type Result = Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRootMessage {}

impl Message for GetRootMessage {
    type Result = Result<ObjectState>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveStateChangeSetMessage {
    pub tx_order: u64,
    pub state_change_set: StateChangeSetExt,
}

impl Message for SaveStateChangeSetMessage {
    type Result = Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStateChangeSetsMessage {
    pub tx_orders: Vec<u64>,
}

impl Message for GetStateChangeSetsMessage {
    type Result = Result<Vec<Option<StateChangeSetExt>>>;
}
