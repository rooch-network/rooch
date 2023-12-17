// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use serde::{Deserialize, Serialize};

use moveos_types::h256::H256;

#[derive(Debug, Serialize, Deserialize)]
pub struct PutBatchMessage {
    pub batch_meta: BatchMeta,
    pub batch: Vec<u8>,

    // TODO add put policy
}

pub struct BatchMeta {
    // each batch maps to a L2 block
    pub block_number: u128,
    // sha3_256 hash of the batch data
    pub batch_hash: H256,
    // signature result of BatchMeta
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutBatchResult {
    // TODO checksum algorithm
    // checksum of the batch data:
    // help to check publication integrity, DA server will have to receive the full data to calculate the checksum
    pub checksum: Vec<u8>,
    // signature result of PutBatchResult
    pub signature: Vec<u8>,
}

impl Message for PutBatchMessage {
    type Result = Result<PutBatchResult>;
}
