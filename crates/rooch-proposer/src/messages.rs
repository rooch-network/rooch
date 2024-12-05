// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::{message::Message, scheduler::timer::TimerTick};
use rooch_types::block::Block;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ProposeBlock {}

impl Message for ProposeBlock {
    type Result = ();
}

impl TimerTick for ProposeBlock {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlocksMessage {
    pub block_numbers: Vec<u128>,
}

impl Message for GetBlocksMessage {
    type Result = Result<Vec<Option<Block>>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLastestBlockNumberMessage {}

impl Message for GetLastestBlockNumberMessage {
    type Result = Result<u128>;
}
