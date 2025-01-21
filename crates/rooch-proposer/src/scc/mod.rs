// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_store::transaction_store::TransactionStore;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use rooch_store::da_store::DAMetaStore;
use rooch_store::proposer_store::ProposerStore;
use rooch_store::RoochStore;
use rooch_types::block::Block;
use rooch_types::da::batch::BlockSubmitState;
use rooch_types::transaction::LedgerTransaction;

/// State Commitment Chain(SCC) is a chain of transaction state root
/// This SCC is a mirror of the on-chain SCC
pub struct StateCommitmentChain {
    last_proposed_block_number: Option<u128>,
    last_proposed_block_accumulator_root: H256,
    rooch_store: RoochStore,
    moveos_store: MoveOSStore,
}

impl StateCommitmentChain {
    /// Create a new SCC
    pub fn new(rooch_store: RoochStore, moveos_store: MoveOSStore) -> anyhow::Result<Self> {
        Self::repair_last_proposed(rooch_store.clone())?;

        let last_proposed_block_number = rooch_store.get_last_proposed()?;

        let last_proposed_block_accumulator_root: H256 = match last_proposed_block_number {
            Some(last_proposed) => {
                let last_proposed_block_state = rooch_store.get_block_state(last_proposed)?;
                let ledger_tx = get_ledger_tx(
                    rooch_store.clone(),
                    last_proposed_block_state.block_range.tx_order_end,
                )?;
                ledger_tx.sequence_info.tx_accumulator_root
            }
            None => H256::zero(),
        };

        Ok(Self {
            last_proposed_block_number,
            last_proposed_block_accumulator_root,
            rooch_store,
            moveos_store,
        })
    }

    // last_proposed may beyond the DA submitted caused by manual rollback/revert
    // we need to repair the last proposed block number
    // invoke it when new scc is created
    fn repair_last_proposed(rooch_store: RoochStore) -> anyhow::Result<()> {
        let last_proposed_block_number = rooch_store.get_last_proposed()?;
        if last_proposed_block_number.is_none() {
            return Ok(());
        }
        let last_proposed = last_proposed_block_number.unwrap();

        let background_submit_block_cursor = rooch_store.get_background_submit_block_cursor()?;
        match background_submit_block_cursor {
            Some(background_submit_block_cursor) => {
                if background_submit_block_cursor < last_proposed {
                    rooch_store.set_last_proposed(background_submit_block_cursor)?;
                }
            }
            None => {
                rooch_store.clear_last_proposed()?;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn get_block(&self, block_number: u128) -> anyhow::Result<Block> {
        match self.last_proposed_block_number {
            Some(last_proposed) => {
                if block_number > last_proposed {
                    return Err(anyhow::anyhow!(
                        "block: {} is greater than the last proposed block number: {}",
                        block_number,
                        last_proposed,
                    ));
                }
                let block_da_submit_state = self.rooch_store.get_block_state(block_number)?;
                if !block_da_submit_state.done {
                    return Err(anyhow::anyhow!(
                        "block: {} da submit is not done but proposed. database is inconsistent",
                        block_number,
                    ));
                }
                let block_range = block_da_submit_state.block_range;
                let batch_size = block_range.tx_order_end - block_range.tx_order_start + 1;
                let (tx_accumulator_root, tx_state_root) =
                    self.get_roots(block_range.tx_order_end)?;
                let prev_tx_accumulator_root = self.get_prev_accumulator_root(block_number)?;

                Ok(Block::new(
                    block_number,
                    batch_size,
                    block_da_submit_state.batch_hash,
                    prev_tx_accumulator_root,
                    tx_accumulator_root,
                    tx_state_root,
                ))
            }
            None => Err(anyhow::anyhow!("No block has been proposed")),
        }
    }

    // get_roots returns the tx accumulator root & state root of the transaction with the given tx_order
    fn get_roots(&self, tx_order: u64) -> anyhow::Result<(H256, H256)> {
        let mut ledger_tx = get_ledger_tx(self.rooch_store.clone(), tx_order)?;
        let tx_accumulator_root = ledger_tx.sequence_info.tx_accumulator_root;
        let tx_hash = ledger_tx.data.tx_hash();
        let tx_execution_info_opt = self.moveos_store.get_tx_execution_info(tx_hash)?;
        if tx_execution_info_opt.is_none() {
            return Err(anyhow::anyhow!(
                "TransactionExecutionInfo not found for tx_hash: {}",
                tx_hash
            ));
        };
        let tx_state_root = tx_execution_info_opt.unwrap().state_root;
        Ok((tx_accumulator_root, tx_state_root))
    }

    #[allow(dead_code)]
    fn get_accumulator_root(&self, tx_order: u64) -> anyhow::Result<H256> {
        let ledger_tx = get_ledger_tx(self.rooch_store.clone(), tx_order)?;
        Ok(ledger_tx.sequence_info.tx_accumulator_root)
    }

    #[allow(dead_code)]
    fn get_prev_accumulator_root(&self, block_number: u128) -> anyhow::Result<H256> {
        if block_number == 0 {
            return Ok(H256::zero());
        }
        let prev_block_number = block_number - 1;
        let prev_block_da_submit_state = self.rooch_store.get_block_state(prev_block_number)?;
        if !prev_block_da_submit_state.done {
            return Err(anyhow::anyhow!(
                "block: {} da submit is not done but proposed. database is inconsistent",
                block_number,
            ));
        }
        let block_range = prev_block_da_submit_state.block_range;
        let prev_tx_order_end = block_range.tx_order_end;
        self.get_accumulator_root(prev_tx_order_end)
    }

    fn append_new_block(
        &mut self,
        block_da_submit_state: BlockSubmitState,
    ) -> anyhow::Result<Block> {
        let block_number = block_da_submit_state.block_range.block_number;
        let tx_order_end = block_da_submit_state.block_range.tx_order_end;
        let batch_size = tx_order_end - block_da_submit_state.block_range.tx_order_start + 1;
        let (tx_accumulator_root, tx_state_root) = self.get_roots(tx_order_end)?;
        let prev_tx_accumulator_root = self.last_proposed_block_accumulator_root;
        let block = Block::new(
            block_number,
            batch_size,
            block_da_submit_state.batch_hash,
            prev_tx_accumulator_root,
            tx_accumulator_root,
            tx_state_root,
        );
        self.last_proposed_block_number = Some(block_number);
        self.last_proposed_block_accumulator_root = tx_accumulator_root;
        Ok(block)
    }

    pub fn set_last_proposed(&self, block_number: u128) -> anyhow::Result<()> {
        self.rooch_store.set_last_proposed(block_number)
    }

    /// Trigger the proposer to propose a new block
    pub async fn propose_block(&mut self) -> anyhow::Result<Option<Block>> {
        let last_proposed = self.rooch_store.get_last_proposed()?;
        let next_propose_block_number = match last_proposed {
            Some(last_proposed) => last_proposed + 1,
            None => 0,
        };
        let next_block_da_state_opt = self
            .rooch_store
            .try_get_block_state(next_propose_block_number)?; // DB error
        match next_block_da_state_opt {
            Some(next_block_da_state) => {
                if !next_block_da_state.done {
                    Ok(None)
                } else {
                    let block = self.append_new_block(next_block_da_state)?;
                    Ok(Some(block))
                }
            }
            None => {
                // init state, no block state
                Ok(None)
            }
        }
    }
}

fn get_ledger_tx(rooch_store: RoochStore, tx_order: u64) -> anyhow::Result<LedgerTransaction> {
    let tx_opt = rooch_store
        .get_transaction_store()
        .get_tx_by_order(tx_order)?;
    if tx_opt.is_none() {
        return Err(anyhow::anyhow!(
            "LedgerTransaction not found for order: {}",
            tx_order
        ));
    }
    Ok(tx_opt.unwrap())
}
