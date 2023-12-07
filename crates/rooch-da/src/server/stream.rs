// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// TODO move stream extent segment to rooch-derive
// TODO streamer manage extent(segment) stream:
// 1. create extent
// 2. add batch
// 3. submit batch async
// ...

use anyhow::Result;

pub trait StreamT {
    // add batch to the stream
    // batch should be persisted before return ok
    fn add_batch(&self, batch: Vec<u8>) -> Result<()> ;
}