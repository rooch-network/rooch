// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// TODO tx buffer for building batch
// TODO using ticker to trigger submit batch to DA server (reuse proposer ticker)

use crate::client::BatchClient;
use crate::messages::BatchPutRequest;

pub struct Batcher<T: BatchClient> {
    clients: Vec<T>,
}

impl<T: BatchClient> Batcher<T> {
    pub fn new(clients: Vec<T>) -> Self {
        Batcher { clients }
    }

    pub fn submit(&self, request: BatchPutRequest) {
        for client in &self.clients {
            let response = client.put(request.clone());
            // do something with response
        }
    }
}

