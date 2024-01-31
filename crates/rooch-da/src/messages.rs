// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use serde::{Deserialize, Serialize};

use moveos_types::h256::H256;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Batch {
    // each batch maps to a L2 block
    pub block_number: u128,
    // sha3_256 hash of the batch data
    pub batch_hash: H256,
    // encoded tx list
    pub data: Vec<u8>,
}

impl Message for Batch {
    type Result = Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutBatchInternalDAMessage {
    pub batch: Batch,
    // TODO add put policy
}

impl Message for PutBatchInternalDAMessage {
    type Result = Result<()>;
}
