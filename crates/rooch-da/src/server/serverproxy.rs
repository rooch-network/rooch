// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;

use crate::messages::PutBatchInternalDAMessage;

#[async_trait]
pub trait DAServerProxy: Sync + Send {
    async fn public_batch(&self, request: PutBatchInternalDAMessage) -> Result<()>;
}

// DAServerNopProxy is a no-op implementation of DAServerProxy
pub struct DAServerNopProxy;

#[async_trait]
impl DAServerProxy for DAServerNopProxy {
    async fn public_batch(&self, _request: PutBatchInternalDAMessage) -> Result<()> {
        Ok(())
    }
}
