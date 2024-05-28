// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::time::SystemTime;

use crate::messages::{
    GetSequencerOrderMessage, GetTransactionByHashMessage, GetTransactionsByHashMessage,
    GetTxHashsMessage, TransactionSequenceMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::h256::{self, H256};
use rooch_store::transaction_store::TransactionStore;
use rooch_store::RoochStore;
use rooch_types::crypto::{RoochKeyPair, Signature};
use rooch_types::sequencer::SequencerOrder;
use rooch_types::transaction::{LedgerTransaction, LedgerTxData, TransactionSequenceInfo};
use tracing::{debug, info};

pub struct SequencerActor {
    last_order: u64,
    sequencer_key: RoochKeyPair,
    rooch_store: RoochStore,
}

impl SequencerActor {
    pub fn new(sequencer_key: RoochKeyPair, rooch_store: RoochStore) -> Result<Self> {
        let last_order_opt = rooch_store
            .get_meta_store()
            .get_sequencer_order()?
            .map(|order| order.last_order);
        // Reserve tx_order = 0 for genesis tx
        let last_order = last_order_opt.unwrap_or(0u64);
        info!("Load latest sequencer order {:?}", last_order);
        Ok(Self {
            last_order,
            sequencer_key,
            rooch_store,
        })
    }

    pub fn sequence(&mut self, mut tx_data: LedgerTxData) -> Result<LedgerTransaction> {
        let now = SystemTime::now();
        let tx_timestamp = now.duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as u64;

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
        let hash = tx_data.tx_hash();
        let mut witness_data = hash.as_ref().to_vec();
        witness_data.extend(tx_order.to_le_bytes().iter());
        let witness_hash = h256::sha3_256_of(&witness_data);
        let tx_order_signature = Signature::new_hashed(&witness_hash.0, &self.sequencer_key)
            .as_ref()
            .to_vec();
        let tx =
            Self::build_ledger_transaction(tx_data, tx_timestamp, tx_order, tx_order_signature)?;

        self.rooch_store.save_transaction(tx.clone())?;
        debug!("sequencer tx: {} order: {:?}", hash, tx_order);

        Ok(tx)
    }

    pub fn build_ledger_transaction(
        tx_data: LedgerTxData,
        tx_timestamp: u64,
        tx_order: u64,
        tx_order_signature: Vec<u8>,
    ) -> Result<LedgerTransaction> {
        let tx_accumulator_root = H256::random();
        let tx_sequence_info = TransactionSequenceInfo {
            tx_order,
            tx_order_signature,
            tx_accumulator_root,
            tx_timestamp,
        };

        let ledger_tx = LedgerTransaction::new(tx_data, tx_sequence_info);
        Ok(ledger_tx)
    }
}

impl Actor for SequencerActor {}

#[async_trait]
impl Handler<TransactionSequenceMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: TransactionSequenceMessage,
        _ctx: &mut ActorContext,
    ) -> Result<LedgerTransaction> {
        self.sequence(msg.tx)
    }
}

#[async_trait]
impl Handler<GetTransactionByHashMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTransactionByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<LedgerTransaction>> {
        self.rooch_store.get_transaction_by_hash(msg.hash)
    }
}

#[async_trait]
impl Handler<GetTransactionsByHashMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTransactionsByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<LedgerTransaction>>> {
        self.rooch_store.get_transactions_by_hash(msg.tx_hashes)
    }
}

#[async_trait]
impl Handler<GetTxHashsMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTxHashsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<H256>>> {
        let GetTxHashsMessage { tx_orders } = msg;
        self.rooch_store.get_tx_hashs(tx_orders)
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
