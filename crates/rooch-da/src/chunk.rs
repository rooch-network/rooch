// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;
use serde::Serialize;

// DABatchV0 is the first version of the batch inside a chunk, each batch is a chunk
#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct DABatchV0 {
    pub version: u8,
    // each batch maps to a L2 block
    pub block_number: u128,
    // sha3_256 hash of the batch data
    pub batch_hash: H256,
    // encoded tx list
    pub data: Vec<u8>,
}
