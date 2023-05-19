// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use rooch_types::transaction::{authenticator::Authenticator, TypedTransaction};

/// Transaction Sequence Message
#[derive(Debug)]
pub struct TransactionSequenceMessage {
    pub tx: TypedTransaction,
}

#[derive(Debug)]
pub struct TransactionSequenceResult {
    /// The tx order
    pub tx_order: u128,
    /// The tx order signature, it is the signature of the sequencer to commit the tx order.
    //TODO confirm the type.
    pub tx_order_signature: Authenticator,
}

impl Message for TransactionSequenceMessage {
    type Result = Result<TransactionSequenceResult>;
}
