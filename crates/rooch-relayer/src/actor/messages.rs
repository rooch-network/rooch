// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bitcoincore_rpc::json;
use coerce::actor::{message::Message, scheduler::timer::TimerTick};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct RelayTick {}

impl Message for RelayTick {
    type Result = ();
}

impl TimerTick for RelayTick {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlockMessage {
    pub hash: bitcoin::BlockHash,
}

impl Message for GetBlockMessage {
    type Result = Result<bitcoin::Block>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBestBlockHashMessage {}

impl Message for GetBestBlockHashMessage {
    type Result = Result<bitcoin::BlockHash>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlockHashMessage {
    pub height: u64,
}

impl Message for GetBlockHashMessage {
    type Result = Result<bitcoin::BlockHash>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlockHeaderInfoMessage {
    pub hash: bitcoin::BlockHash,
}

impl Message for GetBlockHeaderInfoMessage {
    type Result = Result<json::GetBlockHeaderResult>;
}
