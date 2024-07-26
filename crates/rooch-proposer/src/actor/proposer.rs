// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use prometheus::Registry;
use std::sync::Arc;

use crate::metrics::ProposerMetrics;
use rooch_da::proxy::DAProxy;
use rooch_types::crypto::RoochKeyPair;

use crate::scc::StateCommitmentChain;

use super::messages::{ProposeBlock, TransactionProposeMessage, TransactionProposeResult};

const TRANSACTION_PROPOSE_FN_NAME: &str = "transaction_propose";
const PROPOSE_BLOCK_FN_NAME: &str = "propose_block";

pub struct ProposerActor {
    proposer_key: RoochKeyPair,
    scc: StateCommitmentChain,
    metrics: Arc<ProposerMetrics>,
}

impl ProposerActor {
    pub fn new(proposer_key: RoochKeyPair, da_proxy: DAProxy, registry: &Registry) -> Self {
        Self {
            proposer_key,
            scc: StateCommitmentChain::new(da_proxy),
            metrics: Arc::new(ProposerMetrics::new(registry)),
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
        let fn_name = TRANSACTION_PROPOSE_FN_NAME;
        let _timer = self
            .metrics
            .proposer_transaction_propose_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        self.scc.append_transaction(msg);
        Ok(TransactionProposeResult {})
    }
}

#[async_trait]
impl Handler<ProposeBlock> for ProposerActor {
    async fn handle(&mut self, _message: ProposeBlock, _ctx: &mut ActorContext) {
        let fn_name = PROPOSE_BLOCK_FN_NAME;
        let _timer = self
            .metrics
            .proposer_propose_block_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
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
        let size = 0u64;
        self.metrics
            .proposer_propose_block_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
    }
}
