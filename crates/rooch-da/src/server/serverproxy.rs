// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;

use crate::messages::{PutBatchMessage, PutBatchResult};

#[async_trait]
pub trait DAServerProxy: Sync + Send {
    async fn put_batch(&self, request: PutBatchMessage) -> Result<PutBatchResult>;
}

// DAServerNopProxy is a no-op implementation of DAServerProxy
pub struct DAServerNopProxy;

#[async_trait]
impl DAServerProxy for DAServerNopProxy {
    async fn put_batch(&self, _request: PutBatchMessage) -> Result<PutBatchResult> {
        Ok(PutBatchResult::default())
    }
}
