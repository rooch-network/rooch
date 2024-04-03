// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::{message::Message, scheduler::timer::TimerTick};
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_types::transaction::LedgerTransaction;

/// Transaction Propose Message
#[derive(Debug)]
pub struct TransactionProposeMessage {
    pub tx: LedgerTransaction,
    pub tx_execution_info: TransactionExecutionInfo,
}

#[derive(Debug)]
pub struct TransactionProposeResult {
    //TODO define result
}

impl Message for TransactionProposeMessage {
    type Result = Result<TransactionProposeResult>;
}

#[derive(Clone)]
pub struct ProposeBlock {}

impl Message for ProposeBlock {
    type Result = ();
}

impl TimerTick for ProposeBlock {}
