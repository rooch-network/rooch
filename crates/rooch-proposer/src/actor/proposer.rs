// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{Actor, context::ActorContext, message::Handler};

use rooch_da::proxy::DAProxy;
use rooch_types::crypto::RoochKeyPair;

use crate::scc::StateCommitmentChain;

use super::messages::{ProposeBlock, TransactionProposeMessage, TransactionProposeResult};

pub struct ProposerActor {
    proposer_key: RoochKeyPair,
    scc: StateCommitmentChain,
}

impl ProposerActor {
    pub fn new(proposer_key: RoochKeyPair, da_proxy: DAProxy) -> Self {
        Self {
            proposer_key,
            scc: StateCommitmentChain::new(da_proxy),
        }
    }
}

impl Actor for ProposerActor {}

#[async_trait]
impl Handler<TransactionProposeMessage> for ProposerActor {
    async fn handle(
        &mut self,
        msg: TransactionProposeMessage,
        _ctx: &mut ActorContext,
    ) -> Result<TransactionProposeResult> {
        self.scc.append_transaction(msg);
        Ok(TransactionProposeResult {})
    }
}

#[async_trait]
impl Handler<ProposeBlock> for ProposerActor {
    async fn handle(&mut self, _message: ProposeBlock, _ctx: &mut ActorContext) {
        let block = self.scc.propose_block().await;
        match block {
            Some(block) => {
                log::info!(
                    "[ProposeBlock] block_number: {}, batch_size: {:?}",
                    block.block_number,
                    block.batch_size
                );
            }
            None => {
                log::debug!("[ProposeBlock] no transaction to propose block");
            }
        };
        //TODO submit to the on-chain SCC contract use the proposer key
        let _proposer_key = &self.proposer_key;
    }
}
