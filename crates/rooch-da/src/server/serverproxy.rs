// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;

use crate::messages::{PutBatchMessage, PutBatchResult};

pub trait DAServerProxy {
    fn put_batch(&self, request: PutBatchMessage) -> Result<PutBatchResult>;
}

// DAServerNopProxy is a no-op implementation of DAServerProxy
pub struct DAServerNopProxy;

impl DAServerProxy for DAServerNopProxy {
    fn put_batch(&self, _request: PutBatchMessage) -> Result<PutBatchResult> {
        Ok(PutBatchResult::default())
    }
}