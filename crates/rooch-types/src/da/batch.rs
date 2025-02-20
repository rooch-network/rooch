// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::crypto::{RoochKeyPair, RoochSignature, Signature};
use crate::transaction::LedgerTransaction;
use fastcrypto::traits::ToFromBytes;
use moveos_types::h256;
use moveos_types::h256::{sha2_256_of, H256};
use serde::{Deserialize, Serialize};

/// The tx order range of the block.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct BlockRange {
    /// The Rooch block number for DA, each batch maps to a block
    pub block_number: u128,
    /// The start tx order of the block (inclusive)
    pub tx_order_start: u64,
    /// The end tx order of the block (inclusive)
    pub tx_order_end: u64,
}

impl BlockRange {
    pub fn is_legal(&self, last_order: u64) -> bool {
        self.tx_order_start <= self.tx_order_end && self.tx_order_end <= last_order
    }
}

/// The state of the block submission.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct BlockSubmitState {
    /// tx order range of the block
    pub block_range: BlockRange,
    /// DABatch::get_hash()
    pub batch_hash: H256,
    /// submitted or not
    pub done: bool,
}

impl BlockSubmitState {
    /// Create a new BlockSubmitState
    pub fn new(block_number: u128, tx_order_start: u64, tx_order_end: u64) -> Self {
        Self {
            block_range: BlockRange {
                block_number,
                tx_order_start,
                tx_order_end,
            },
            batch_hash: H256::zero(),
            done: false,
        }
    }
    pub fn new_done(
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        batch_hash: H256,
    ) -> Self {
        Self {
            block_range: BlockRange {
                block_number,
                tx_order_start,
                tx_order_end,
            },
            batch_hash,
            done: true,
        }
    }
}

/// Meta of DA batch
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DABatchMeta {
    /// tx order range of the block
    pub block_range: BlockRange,
    /// sha256h of encoded tx_list
    pub tx_list_hash: H256,
}

impl DABatchMeta {
    pub fn new(
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        tx_list_hash: H256,
    ) -> Self {
        Self {
            block_range: BlockRange {
                block_number,
                tx_order_start,
                tx_order_end,
            },
            tx_list_hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SignedDABatchMeta {
    pub meta: DABatchMeta,
    pub signature: Vec<u8>,
}

/// A batch is a collection of transactions. It is the unit of data flow in DA Stream
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DABatch {
    /// The metadata of the batch
    pub meta: DABatchMeta,
    /// meta signature, signed by sequencer.
    pub meta_signature: Vec<u8>,
    /// encoded Vec<LedgerTransaction>
    pub tx_list_bytes: Vec<u8>,
}

impl DABatch {
    pub fn new(
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        tx_list: &Vec<LedgerTransaction>,
        sequencer_key: &RoochKeyPair,
    ) -> anyhow::Result<Self> {
        // Verify transaction ordering constraints before signing
        verify_tx_order(block_number, tx_list, tx_order_start, tx_order_end)?;

        let tx_list_bytes = bcs::to_bytes(tx_list).expect("encode tx_list should success");
        let tx_list_hash = sha2_256_of(&tx_list_bytes);
        let batch_meta = DABatchMeta::new(block_number, tx_order_start, tx_order_end, tx_list_hash);
        let meta_bytes = bcs::to_bytes(&batch_meta).expect("encode batch_meta should success");
        let meta_hash = sha2_256_of(&meta_bytes);
        let meta_signature = Signature::sign(&meta_hash.0, sequencer_key)
            .as_ref()
            .to_vec();

        Ok(Self {
            meta: batch_meta,
            meta_signature,
            tx_list_bytes,
        })
    }

    pub fn get_hash(&self) -> H256 {
        let meta_bytes = bcs::to_bytes(&self.meta).expect("encode batch_meta should success");
        sha2_256_of(&meta_bytes)
    }

    /// Verify the batch, helpful when unpacking a batch from DA or other sources.
    /// If verify_order is true, it will verify the tx order and its signature
    pub fn verify(&self, verify_order: bool) -> anyhow::Result<()> {
        // verify meta
        let tx_list_bytes = &self.tx_list_bytes;
        let tx_list_hash = sha2_256_of(tx_list_bytes);
        if tx_list_hash != self.meta.tx_list_hash {
            return Err(anyhow::anyhow!("tx_list_hash mismatch"));
        }
        let batch_hash = self.get_hash();
        let meta_signature = Signature::from_bytes(&self.meta_signature)?;
        meta_signature.verify(batch_hash.as_bytes())?;
        // verify order and signature
        if verify_order {
            self.verify_order_and_signature()?;
        }

        Ok(())
    }

    fn verify_order_and_signature(&self) -> anyhow::Result<()> {
        let tx_list = self.get_tx_list()?;
        verify_tx_order(
            self.meta.block_range.block_number,
            &tx_list,
            self.meta.block_range.tx_order_start,
            self.meta.block_range.tx_order_end,
        )?;
        for mut tx in tx_list {
            let tx_order = tx.sequence_info.tx_order;
            let tx_hash = tx.data.tx_hash();
            let mut witness_data = tx_hash.as_ref().to_vec();
            witness_data.extend(tx_order.to_le_bytes().iter());
            let witness_hash = h256::sha3_256_of(&witness_data);
            let tx_order_signature = Signature::from_bytes(&tx.sequence_info.tx_order_signature)?;
            tx_order_signature.verify(witness_hash.as_bytes())?;
        }

        Ok(())
    }

    pub fn get_tx_list(&self) -> anyhow::Result<Vec<LedgerTransaction>> {
        let tx_list: Vec<LedgerTransaction> = bcs::from_bytes(&self.tx_list_bytes)?;
        Ok(tx_list)
    }
}

// fast order verification
fn verify_tx_order(
    block_number: u128,
    tx_list: &Vec<LedgerTransaction>,
    tx_order_start: u64,
    tx_order_end: u64,
) -> anyhow::Result<()> {
    let mut exp_tx_order = tx_order_start;
    for tx in tx_list {
        let tx_order = tx.sequence_info.tx_order;
        if tx_order != exp_tx_order {
            return Err(anyhow::anyhow!(
                "Transaction order is not strictly incremental for block {}: exp_tx_order: {}, tx_order: {}",
                block_number, exp_tx_order, tx_order
            ));
        }
        exp_tx_order += 1;
    }
    exp_tx_order -= 1;
    if exp_tx_order != tx_order_end {
        return Err(anyhow::anyhow!(
            "Transaction order is not strictly incremental for block {}: exp_tx_order: {}, tx_order_end: {}",
            block_number, exp_tx_order, tx_order_end
        ));
    }

    Ok(())
}
