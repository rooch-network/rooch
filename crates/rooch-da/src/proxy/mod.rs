// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::ActorRef;

use crate::actor::da::DAActor;
use crate::messages::Batch;

#[derive(Clone)]
pub struct DAProxy {
    pub actor: ActorRef<DAActor>,
}

impl DAProxy {
    pub fn new(actor: ActorRef<DAActor>) -> Self {
        Self { actor }
    }

    pub async fn submit_batch(
        &self,
        batch: Batch,
    ) -> anyhow::Result<()> {
        self
            .actor
            .send(batch)
            .await?
    }
}