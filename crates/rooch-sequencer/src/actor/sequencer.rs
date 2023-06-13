// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{
    TransactionByHashMessage, TransactionByIndexMessage, TransactionSequenceMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::h256;
use rooch_store::RoochDB;
use rooch_types::{
    crypto::{RoochKeyPair, Signature},
    transaction::AbstractTransaction,
};
use rooch_types::{
    transaction::{TransactionSequenceInfo, TypedTransaction},
    H256,
};

pub struct SequencerActor {
    last_order: u128,
    sequencer_key: RoochKeyPair,
    rooch_db: RoochDB,
}

impl SequencerActor {
    pub fn new(sequencer_key: RoochKeyPair, rooch_db: RoochDB) -> Self {
        Self {
            last_order: 0,
            sequencer_key,
            rooch_db,
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
        let hash = tx.tx_hash();
        let mut witness_data = hash.as_ref().to_vec();
        witness_data.extend(tx_order.to_le_bytes().iter());
        let witness_hash = h256::sha3_256_of(&witness_data);
        let tx_order_signature = Signature::new_hashed(&witness_hash.0, &self.sequencer_key).into();
        self.last_order = tx_order;

        self.rooch_db.transaction_store.add(tx);

        let tx_accumulator_root = H256::random();
        Ok(TransactionSequenceInfo {
            tx_order,
            tx_order_signature,
            tx_accumulator_root,
        })
    }
}

#[async_trait]
impl Handler<TransactionByHashMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: TransactionByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<TypedTransaction>> {
        Ok(self.rooch_db.transaction_store.get_by_hash(msg.hash))
    }
}

#[async_trait]
impl Handler<TransactionByIndexMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: TransactionByIndexMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<TypedTransaction>> {
        Ok(self
            .rooch_db
            .transaction_store
            .get_by_index(msg.start, msg.limit))
    }
}
