// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::TransactionSequenceMessage;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::h256;
use rooch_types::{
    transaction::{authenticator::AccountPrivateKey, TransactionSequenceInfo},
    H256,
};

pub struct SequencerActor {
    last_order: u128,
    sequencer_key: AccountPrivateKey,
}

impl SequencerActor {
    pub fn new(sequencer_key: AccountPrivateKey) -> Self {
        Self {
            last_order: 0,
            sequencer_key,
        }
    }
}

impl Actor for SequencerActor {}

#[async_trait]
impl Handler<TransactionSequenceMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: TransactionSequenceMessage,
        _ctx: &mut ActorContext,
    ) -> Result<TransactionSequenceInfo> {
        let tx = msg.tx;
        let tx_order = self.last_order + 1;
        let hash = tx.hash();
        let mut witness_data = hash.as_ref().to_vec();
        witness_data.extend(tx_order.to_le_bytes().iter());
        let witness_hash = h256::sha3_256_of(&witness_data);
        let tx_order_signature = self.sequencer_key.sign(witness_hash.as_bytes());
        self.last_order = tx_order;
        //TODO introduce accumulator
        let tx_accumulator_root = H256::random();
        Ok(TransactionSequenceInfo {
            tx_order,
            tx_order_signature,
            tx_accumulator_root,
        })
    }
}
