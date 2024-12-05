// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::proposer::ProposerActor;
use crate::messages::{GetBlocksMessage, GetLastestBlockNumberMessage};
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_types::block::Block;

#[derive(Clone)]
pub struct ProposerProxy {
    pub actor: ActorRef<ProposerActor>,
}

impl ProposerProxy {
    pub fn new(actor: ActorRef<ProposerActor>) -> Self {
        Self { actor }
    }

    pub async fn get_blocks(&self, block_numbers: Vec<u128>) -> Result<Vec<Option<Block>>> {
        self.actor.send(GetBlocksMessage { block_numbers }).await?
    }

    pub async fn latest_block_number(&self) -> Result<u128> {
        self.actor.send(GetLastestBlockNumberMessage {}).await?
    }
}
