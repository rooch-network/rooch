// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::server::stream::StreamT;
use anyhow::Result;

struct Stream {
    // TODO
}

impl Stream{
    fn new() -> Self {
        todo!()
    }
}

impl StreamT for Stream {
    fn add_batch(&self, batch: Vec<u8>) -> Result<()> {
        todo!()
    }
}