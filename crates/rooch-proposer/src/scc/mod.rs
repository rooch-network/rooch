// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use moveos_types::h256;
use moveos_types::h256::H256;
use rooch_da::messages::Batch;
use rooch_da::proxy::DAProxy;
use rooch_types::block::Block;
use rooch_types::transaction::AbstractTransaction;

use crate::actor::messages::TransactionProposeMessage;

/// State Commitment Chain(SCC) is a chain of transaction state root
/// This SCC is a mirror of the on-chain SCC
pub struct StateCommitmentChain {
    //TODO save to the storage
    blocks: BTreeMap<u128, Block>,
    buffer: Vec<TransactionProposeMessage>,
    da: DAProxy,
}

impl StateCommitmentChain {
    /// Create a new SCC
    pub fn new(da_proxy: DAProxy) -> Self {
        Self {
            blocks: BTreeMap::new(),
            buffer: Vec::new(),
            da: da_proxy,
        }
    }

    pub fn append_transaction(&mut self, tx: TransactionProposeMessage) {
        self.buffer.push(tx);
    }

    /// Append a new block to the SCC
    fn append_block(&mut self, block: Block) {
        self.blocks.insert(block.block_number, block);
    }

    /// Get the last block of the SCC
    pub fn last_block(&self) -> Option<&Block> {
        self.blocks.values().last()
    }

    /// Get the last block number of the SCC
    pub fn last_block_number(&self) -> Option<u128> {
        self.blocks.keys().last().copied()
    }

    /// Trigger the proposer to propose a new block
    pub async fn propose_block(&mut self) -> Option<&Block> {
        if self.buffer.is_empty() {
            return None;
        }
        // construct a new block from buffer
        let latest_transaction = self.buffer.last().expect("buffer must not empty");
        let tx_accumulator_root = latest_transaction.tx_sequence_info.tx_accumulator_root;
        let state_roots = self
            .buffer
            .iter()
            .map(|tx| tx.tx_execution_info.state_root)
            .collect();

        let batch_size = self.buffer.len() as u64;
        let last_block = self.last_block();
        let (block_number, prev_tx_accumulator_root) = match last_block {
            Some(block) => {
                let block_number = block.block_number + 1;
                let prev_tx_accumulator_root = block.tx_accumulator_root;
                (block_number, prev_tx_accumulator_root)
            }
            None => {
                let block_number = 0;
                let prev_tx_accumulator_root = H256::zero();
                (block_number, prev_tx_accumulator_root)
            }
        };

        // submit batch to DA server
        // TODO move batch submit out of proposer
        let batch_data: Vec<u8> = self.buffer.iter().flat_map(|tx| tx.tx.encode()).collect();
        // regard batch(tx list) as a blob: easy to check integrity
        let batch_hash = h256::sha3_256_of(&batch_data);
        if let Err(e) = self
            .da
            .submit_batch(Batch {
                block_number,
                batch_hash,
                data: batch_data,
            })
            .await
        {
            log::error!("submit batch to DA server failed: {}", e);
            return None;
        }

        let new_block = Block::new(
            block_number,
            batch_size,
            prev_tx_accumulator_root,
            tx_accumulator_root,
            state_roots,
        );
        self.append_block(new_block);
        self.buffer.clear();
        self.last_block()
    }
}
