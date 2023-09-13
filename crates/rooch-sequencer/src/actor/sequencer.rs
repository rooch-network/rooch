// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{
    GetTransactionsMessage, TransactionByHashMessage, TransactionSequenceMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::h256;
use rooch_store::meta_store::MetaStore;
use rooch_store::transaction_store::TransactionStore;
use rooch_store::RoochStore;
use rooch_types::sequencer::SequencerOrder;
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
    rooch_store: RoochStore,
}

impl SequencerActor {
    pub fn new(
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
        is_genesis: bool,
    ) -> Result<Self> {
        let last_order = rooch_store
            .get_meta_store()
            .get_sequencer_order()?
            .map(|order| order.last_order);
        let last_order = if is_genesis {
            last_order.unwrap_or(0u128)
        } else {
            return Err(anyhow::anyhow!("Invalid sequencer order"));
        };

        Ok(Self {
            last_order,
            sequencer_key,
            rooch_store,
        })
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

        let _ = self.rooch_store.save_transaction(tx).map_err(|e| {
            anyhow::anyhow!(
                "TransactionSequenceMessage handler save transaction failed: {}",
                e
            )
        });
        let _ = self
            .rooch_store
            .save_tx_seq_info_mapping(tx_order, hash)
            .map_err(|e| {
                anyhow::anyhow!(
                    "TransactionSequenceMessage handler save tx seq mapping failed: {}",
                    e
                )
            });

        let _ = self
            .rooch_store
            .save_sequencer_order(SequencerOrder::new(self.last_order))?;

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
        self.rooch_store.get_tx_by_hash(msg.hash)
    }
}

#[async_trait]
impl Handler<GetTransactionsMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTransactionsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<TypedTransaction>>> {
        self.rooch_store.get_transactions(msg.tx_hashes)
    }
}
