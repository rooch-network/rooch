// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use moveos_types::moveos_std::tx_context::TxContext;
use rooch_types::{
    address::BitcoinAddress,
    transaction::{ExecuteTransactionResponse, L1BlockWithBody, RoochTransaction},
};

#[derive(Clone)]
pub struct ExecuteL2TxMessage {
    pub tx: RoochTransaction,
}

impl Message for ExecuteL2TxMessage {
    type Result = Result<ExecuteTransactionResponse>;
}

#[derive(Clone)]
pub struct ExecuteL1BlockMessage {
    pub ctx: TxContext,
    pub tx: L1BlockWithBody,
    pub sequencer_address: BitcoinAddress,
}

impl Message for ExecuteL1BlockMessage {
    type Result = Result<ExecuteTransactionResponse>;
}
