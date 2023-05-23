// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::{message::Message, scheduler::timer::TimerTick};
use rooch_types::transaction::{TransactionInfo, TransactionSequenceInfo, TypedTransaction};

/// Transaction Sequence Message
#[derive(Debug)]
pub struct TransactionSequenceMessage {
    pub tx: TypedTransaction,
}

/// Transaction Propose Message
#[derive(Debug)]
pub struct TransactionProposeMessage {
    pub tx: TypedTransaction,
    pub tx_execution_info: TransactionInfo,
    pub tx_sequence_info: TransactionSequenceInfo,
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
