// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;

// TODO use actor
// Request to put batch to the DA server
pub struct BatchPutRequest {
    // RPC version
    pub version: u8,
    // each batch maps to a L2 block
    pub block_number: u128,
    // sha3_256 hash of the batch data
    pub batch_hash: H256,
    pub batch: Vec<u8>,

    // TODO add put policy

    // signature result of BatchPutRequest
    pub signature: Vec<u8>,
}

// Response to put batch to the DA server
pub struct BatchPutResponse {
    // RPC version, double check with request
    pub version: u8,
    // TODO checksum algorithm
    // checksum of the batch data: help to check publication integrity
    pub checksum: Vec<u8>,

    // signature result of BatchPutResponse
    pub signature: Vec<u8>,

}