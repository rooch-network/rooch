// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{
    GetSequencerOrderMessage, GetTransactionByHashMessage, GetTransactionsByHashMessage,
    GetTxSequenceInfoMappingByHashMessage, GetTxSequenceInfoMappingByOrderMessage,
    GetTxSequenceInfosMessage, TransactionSequenceMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::h256;
use rooch_store::meta_store::MetaStore;
use rooch_store::transaction_store::TransactionStore;
use rooch_store::RoochStore;
use rooch_types::sequencer::SequencerOrder;
use rooch_types::transaction::TransactionSequenceInfoMapping;
use rooch_types::{
    crypto::{RoochKeyPair, Signature},
    transaction::AbstractTransaction,
};
use rooch_types::{
    transaction::{TransactionSequenceInfo, TypedTransaction},
    H256,
};
use tracing::info;

pub struct SequencerActor {
    last_order: u128,
    sequencer_key: RoochKeyPair,
    rooch_store: RoochStore,
}

impl SequencerActor {
    pub fn new(
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
        _is_genesis: bool,
    ) -> Result<Self> {
        let last_order_opt = rooch_store
            .get_meta_store()
            .get_sequencer_order()?
            .map(|order| order.last_order);
        let last_order = last_order_opt.unwrap_or(0u128);
        info!("Load latest sequencer order {:?}", last_order);
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
        let tx_order = if self.last_order == 0 {
            let last_order_opt = self
                .rooch_store
                .get_meta_store()
                .get_sequencer_order()?
                .map(|order| order.last_order);
            match last_order_opt {
                Some(last_order) => last_order + 1,
                None => 0,
            }
        } else {
            self.last_order + 1
        };
        let hash = tx.tx_hash();
        let mut witness_data = hash.as_ref().to_vec();
        witness_data.extend(tx_order.to_le_bytes().iter());
        let witness_hash = h256::sha3_256_of(&witness_data);
        let tx_order_signature = Signature::new_hashed(&witness_hash.0, &self.sequencer_key).into();
        self.last_order = tx_order;

        self.rooch_store.save_transaction(tx)?;
        self.rooch_store
            .save_tx_sequence_info_mapping(tx_order, hash)?;
        self.rooch_store
            .save_tx_sequence_info_reverse_mapping(hash, tx_order)?;

        self.rooch_store
            .save_sequencer_order(SequencerOrder::new(self.last_order))?;

        let tx_accumulator_root = H256::random();
        let tx_sequence_info = TransactionSequenceInfo {
            tx_order,
            tx_order_signature,
            tx_accumulator_root,
        };
        self.rooch_store
            .save_tx_sequence_info(tx_sequence_info.clone())?;
        Ok(tx_sequence_info)
    }
}

#[async_trait]
impl Handler<GetTransactionByHashMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTransactionByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<TypedTransaction>> {
        self.rooch_store.get_transaction_by_hash(msg.hash)
    }
}

#[async_trait]
impl Handler<GetTransactionsByHashMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTransactionsByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<TypedTransaction>>> {
        self.rooch_store.get_transactions_by_hash(msg.tx_hashes)
    }
}

#[async_trait]
impl Handler<GetTxSequenceInfoMappingByOrderMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTxSequenceInfoMappingByOrderMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<TransactionSequenceInfoMapping>>> {
        let GetTxSequenceInfoMappingByOrderMessage { tx_orders } = msg;
        self.rooch_store
            .get_transaction_store()
            .get_tx_sequence_info_mapping_by_order(tx_orders)
    }
}

#[async_trait]
impl Handler<GetTxSequenceInfoMappingByHashMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTxSequenceInfoMappingByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<TransactionSequenceInfoMapping>>> {
        let GetTxSequenceInfoMappingByHashMessage { tx_hashes } = msg;
        self.rooch_store
            .get_transaction_store()
            .multi_get_tx_sequence_info_mapping_by_hash(tx_hashes)
    }
}

#[async_trait]
impl Handler<GetTxSequenceInfosMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTxSequenceInfosMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<TransactionSequenceInfo>>> {
        let GetTxSequenceInfosMessage { orders } = msg;
        self.rooch_store
            .get_transaction_store()
            .get_tx_sequence_infos(orders)
    }
}

#[async_trait]
impl Handler<GetSequencerOrderMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetSequencerOrderMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<SequencerOrder>> {
        let GetSequencerOrderMessage {} = msg;
        self.rooch_store.get_meta_store().get_sequencer_order()
    }
}
