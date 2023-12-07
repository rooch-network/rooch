// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0


// TODO get request and response
// 1. get by block number
// 2. get by batch hash
// 3. pull by stream
//
// TODO ECC for SDC protection (wrong response attacks)

use crate::messages::{BatchPutRequest, BatchPutResponse};

// client interface that make requests of batch to DA server
// TODO async trait
pub trait BatchClient {
    fn put(&self, request: BatchPutRequest) -> BatchPutResponse;
}

// NopBatchClient is a client that does nothing
pub struct NopBatchClient {}

impl BatchClient for NopBatchClient {
    fn put(&self, request: BatchPutRequest) -> BatchPutResponse {
        BatchPutResponse::new()
    }
}