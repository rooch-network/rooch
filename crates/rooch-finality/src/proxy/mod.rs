// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::finality::FinalityActor;
use crate::messages::FinalityMessage;
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_types::finality_block::Block;

#[derive(Clone)]
pub struct FinalityProxy {
    pub actor: ActorRef<FinalityActor>,
}

impl FinalityProxy {
    pub fn new(actor: ActorRef<FinalityActor>) -> Self {
        Self { actor }
    }

    pub async fn finality(&self, block: Block) -> Result<()> {
        self.actor.send(FinalityMessage { block }).await?
    }
}
