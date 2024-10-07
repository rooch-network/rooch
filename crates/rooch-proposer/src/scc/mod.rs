// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::TransactionProposeMessage;
use moveos_types::h256;
use moveos_types::h256::H256;
use rooch_da::actor::messages::PutDABatchMessage;
use rooch_da::proxy::DAServerProxy;
use rooch_types::block::Block;
use rooch_types::da::batch::DABatch;

/// State Commitment Chain(SCC) is a chain of transaction state root
/// This SCC is a mirror of the on-chain SCC
pub struct StateCommitmentChain {
    //TODO save to the storage
    last_block: Option<Block>,
    buffer: Vec<TransactionProposeMessage>,
    da: DAServerProxy,
}

impl StateCommitmentChain {
    /// Create a new SCC
    pub fn new(da_proxy: DAServerProxy) -> Self {
        Self {
            last_block: None,
            buffer: Vec::new(),
            da: da_proxy,
        }
    }

    pub fn append_transaction(&mut self, tx: TransactionProposeMessage) {
        self.buffer.push(tx);
    }

    /// Update last block of the SCC
    fn update_last_block(&mut self, block: Block) {
        self.last_block = Some(block);
    }

    /// Get the last block of the SCC
    pub fn last_block(&self) -> Option<&Block> {
        self.last_block.as_ref()
    }

    /// Get the last block number of the SCC
    pub fn last_block_number(&self) -> Option<u128> {
        self.last_block.as_ref().map(|block| block.block_number)
    }

    /// Trigger the proposer to propose a new block
    pub async fn propose_block(&mut self) -> Option<&Block> {
        if self.buffer.is_empty() {
            return None;
        }
        // construct a new block from buffer
        let latest_transaction = self.buffer.last().expect("buffer must not empty");
        let tx_accumulator_root = latest_transaction.tx.sequence_info.tx_accumulator_root;
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
        let tx_list_bytes: Vec<u8> = self.buffer.iter().flat_map(|tx| tx.tx.encode()).collect();
        let batch_meta = self
            .da
            .pub_batch(PutDABatchMessage {
                tx_order_start: 0,
                tx_order_end: 0,
                tx_list_bytes,
            })
            .await;
        match batch_meta {
            Ok(batch_meta) => {
                log::info!("submit batch to DA success: {:?}", batch_meta);
            }
            Err(e) => {
                log::error!("submit batch to DA failed: {:?}", e);
            }
        }
        // even if the batch submission failed, new block must have been created(otherwise panic)

        // TODO update block struct and add propose logic
        let new_block = Block::new(
            block_number,
            batch_size,
            prev_tx_accumulator_root,
            tx_accumulator_root,
            state_roots,
        );
        self.update_last_block(new_block);
        self.buffer.clear();
        self.last_block()
    }
}
