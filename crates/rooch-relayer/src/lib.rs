// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use rooch_types::transaction::L1BlockWithBody;

pub mod actor;

#[async_trait]
pub trait Relayer: Send + Sync {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    async fn relay(&mut self) -> Result<Option<L1BlockWithBody>>;
}
