// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use rooch_types::transaction::{ExecuteTransactionResponse, RoochTransaction};

#[derive(Clone)]
pub struct ExecuteTransactionMessage {
    pub tx: RoochTransaction,
}

impl Message for ExecuteTransactionMessage {
    type Result = Result<ExecuteTransactionResponse>;
}
