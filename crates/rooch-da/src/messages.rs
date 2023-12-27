// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::message::Message;
use serde::{Deserialize, Serialize};

use moveos_types::h256::H256;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchMeta {
    // each batch maps to a L2 block
    pub block_number: u128,
    // sha3_256 hash of the batch data
    pub batch_hash: H256,
    // TODO use sequencer keypair to sign
    // signature result of BatchMeta
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Batch {
    pub meta: BatchMeta,
    pub data: Vec<u8>,
}

impl Message for Batch {
    type Result = anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutBatchMessage {
    pub batch: Batch,
    // TODO add put policy
}

impl Message for PutBatchMessage {
    type Result = anyhow::Result<PutBatchResult>;
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PutBatchResult {
    // TODO checksum algorithm
    // checksum of the batch data:
    // help to check publication integrity, DA server will have to receive the full data to calculate the checksum
    pub checksum: Vec<u8>,
    // signature result of PutBatchResult
    pub signature: Vec<u8>,
}
