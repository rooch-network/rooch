// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME, DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME};
use moveos_common::utils::to_bytes;
use moveos_types::h256::H256;
use raw_store::rocks::batch::{WriteBatch, WriteBatchCF};
use raw_store::traits::DBStore;
use raw_store::{derive_store, CodecKVStore, SchemaStore, WriteOp};
use rooch_types::da::batch::{BlockRange, BlockSubmitState};
use std::cmp::{min, Ordering};

pub const SUBMITTING_BLOCKS_PAGE_SIZE: usize = 1024;
pub const MAX_TXS_PER_BLOCK_IN_FIX: usize = 8192; // avoid OOM when fix submitting blocks after collapse

// [0,background_submit_block_cursor] are submitted blocks verified by background submitter
pub const BACKGROUND_SUBMIT_BLOCK_CURSOR_KEY: &str = "background_submit_block_cursor";
// for fast access to last block number, must be updated with submitting block state updates atomically
pub const LAST_BLOCK_NUMBER_KEY: &str = "last_block_number";

derive_store!(
    DABlockSubmitStateStore,
    u128,
    BlockSubmitState,
    DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME
);

derive_store!(
    DABlockCursorStore,
    String,
    u128,
    DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME
);

pub trait DAMetaStore {
    // repair da meta: repair tx orders and blocks return (issues, fixed)
    // try to repair blocks by last tx order at starting for catching up historical tx before sequencing new tx:
    // 1. last_tx_order is ahead of last_block_number's tx_order_end: appending submitting blocks until last_order(inclusive)
    // 2. last_tx_order is behind last_block_number's tx_order_end: remove blocks which tx_order_end > last_order
    //   (caused by offline rooch-db rollback/revert cmd):
    //   a. remove blocks from last_block_number to the block which tx_order_end is ahead of last_tx_order
    //   b. update last_block_number to the block which tx_order_end is behind of last_order
    //   c. remove background_submit_block_cursor directly, since we could catch up with the last order by background submitter
    // after repair with condition2, we may need to repair with condition1 for the last block(it will be done automatically)
    //
    // If thorough is true, will try to repair the tx orders first, then repair blocks. It's design for deep repair.
    fn try_repair_da_meta(&self, last_order: u64, thorough: bool)
        -> anyhow::Result<(usize, usize)>;

    // append new submitting block with tx_order_start and tx_order_end, return the block_number
    // LAST_BLOCK_NUMBER & block state must be updated atomically, they must be consistent (version >= v0.7.6)
    // warning: not thread safe
    fn append_submitting_block(
        &self,
        tx_order_start: u64,
        tx_order_end: u64,
    ) -> anyhow::Result<u128>;
    // get submitting blocks(block is not submitted) from start_block(inclusive) with expected count until the end of submitting blocks
    // Result<Vec<BlockRange>>: Vec<BlockRange> is sorted by block_number
    fn get_submitting_blocks(
        &self,
        start_block: u128,
        exp_count: Option<usize>,
    ) -> anyhow::Result<Vec<BlockRange>>;
    // set submitting block done, pass tx_order_start and tx_order_end to save extra get operation
    fn set_submitting_block_done(
        &self,
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        batch_hash: H256,
    ) -> anyhow::Result<()>;

    fn set_background_submit_block_cursor(&self, block_cursor: u128) -> anyhow::Result<()>;
    fn get_background_submit_block_cursor(&self) -> anyhow::Result<Option<u128>>;

    fn get_last_block_number(&self) -> anyhow::Result<Option<u128>>;
    // get block state by block_number, must exist for the block_number, otherwise return error
    fn get_block_state(&self, block_number: u128) -> anyhow::Result<BlockSubmitState>;
    // get block state by block_number, return None if not exist
    fn try_get_block_state(&self, block_number: u128) -> anyhow::Result<Option<BlockSubmitState>>;
}

#[derive(Clone)]
pub struct DAMetaDBStore {
    block_submit_state_store: DABlockSubmitStateStore,
    block_cursor_store: DABlockCursorStore,
}

impl DAMetaDBStore {
    pub fn new(instance: raw_store::StoreInstance) -> anyhow::Result<Self> {
        let store = DAMetaDBStore {
            block_submit_state_store: DABlockSubmitStateStore::new(instance.clone()),
            block_cursor_store: DABlockCursorStore::new(instance),
        };
        Ok(store)
    }

    fn append_block_by_repair(
        &self,
        last_block_number: Option<u128>,
        last_order: u64,
    ) -> anyhow::Result<usize> {
        let block_ranges = self.generate_append_blocks(last_block_number, last_order)?;
        let append_count = block_ranges.len();
        self.append_submitting_blocks(block_ranges)?;
        Ok(append_count)
    }

    // rollback to min_removed_block - 1:
    // 1. submit_state: remove blocks from min_removed_block to last_block_number
    // 2. block_cursor: update last_block_number to min_removed_block - 1
    // 3. background_submit_block_cursor: remove directly
    fn inner_rollback(&self, mut remove_blocks: Vec<u128>) -> anyhow::Result<()> {
        if remove_blocks.is_empty() {
            return Ok(());
        }

        remove_blocks.sort();
        let min_block_number_wait_rm = *remove_blocks.first().unwrap();
        let new_last_block_number = if min_block_number_wait_rm == 0 {
            None
        } else {
            Some(min_block_number_wait_rm - 1)
        };
        let last_block_number_wait_rm = *remove_blocks.last().unwrap();

        let inner_store = self.block_submit_state_store.get_store().store();
        let mut cf_batches: Vec<WriteBatchCF> = Vec::new();

        let state_batch = WriteBatchCF::new_with_rows(
            remove_blocks
                .iter()
                .map(|block_number| {
                    let key = to_bytes(&block_number).unwrap();
                    (key, WriteOp::Deletion)
                })
                .collect(),
            DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME.to_string(),
        );
        cf_batches.push(state_batch);
        match new_last_block_number {
            Some(new_last_block_number) => {
                let last_block_batch = WriteBatchCF {
                    batch: WriteBatch::new_with_rows(vec![(
                        to_bytes(LAST_BLOCK_NUMBER_KEY).unwrap(),
                        WriteOp::Value(to_bytes(&new_last_block_number).unwrap()),
                    )]),
                    cf_name: DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME.to_string(),
                };
                cf_batches.push(last_block_batch);
            }
            None => {
                let last_block_batch = WriteBatchCF {
                    batch: WriteBatch::new_with_rows(vec![(
                        to_bytes(LAST_BLOCK_NUMBER_KEY).unwrap(),
                        WriteOp::Deletion,
                    )]),
                    cf_name: DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME.to_string(),
                };
                cf_batches.push(last_block_batch);
            }
        }
        // remove background_submit_block_cursor directly, since we could catch up with the last order by background submitter
        //  will just ignore the blocks that have been submitted
        cf_batches.push(WriteBatchCF {
            batch: WriteBatch::new_with_rows(vec![(
                to_bytes(BACKGROUND_SUBMIT_BLOCK_CURSOR_KEY).unwrap(),
                WriteOp::Deletion,
            )]),
            cf_name: DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME.to_string(),
        });

        inner_store.write_cf_batch(cf_batches, true)?;
        tracing::info!(
            "rollback to block {:?} successfully, removed blocks: [{},{}]",
            min_block_number_wait_rm,
            remove_blocks,
            last_block_number_wait_rm
        );
        Ok(())
    }

    // generate the blocks need to be removed by tx_order_end > last_order
    pub(crate) fn generate_remove_blocks_after_order(
        &self,
        last_block_number: Option<u128>,
        last_order: u64,
    ) -> anyhow::Result<Vec<u128>> {
        let mut blocks = Vec::new();

        if let Some(mut block_number) = last_block_number {
            loop {
                // last_block_number -> 0, backwards searching
                let block_state = self.get_block_state(block_number)?;
                let block_range = block_state.block_range;

                if block_range.tx_order_end > last_order {
                    blocks.push(block_number);
                } else {
                    break;
                }
                if block_number == 0 {
                    break;
                }
                block_number -= 1;
            }
        }

        Ok(blocks)
    }

    fn get_block_state_opt(&self, block_number: u128) -> anyhow::Result<Option<BlockSubmitState>> {
        self.block_submit_state_store.kv_get(block_number)
    }

    // generate the block ranges to catch up with the last order
    pub(crate) fn generate_append_blocks(
        &self,
        last_block_number: Option<u128>,
        last_order: u64,
    ) -> anyhow::Result<Vec<BlockRange>> {
        // each block has n txs, n = [1, MAX_TXS_PER_BLOCK_IN_FIX], so we need to split txs into multiple blocks
        let mut blocks = Vec::new();
        let mut block_number: u128 = 0;
        let mut tx_order_start: u64 = 1; // tx_order_start starts from 1 (bypass genesis_tx)
        let mut tx_order_end: u64 = min(MAX_TXS_PER_BLOCK_IN_FIX as u64, last_order);
        if let Some(last_block_number) = last_block_number {
            let last_block_state = self.get_block_state(last_block_number)?;
            let last_range = last_block_state.block_range;
            assert!(last_range.tx_order_end < last_order);
            block_number = last_block_number + 1;
            tx_order_start = last_range.tx_order_end + 1;
            tx_order_end = min(
                tx_order_start + MAX_TXS_PER_BLOCK_IN_FIX as u64 - 1,
                last_order,
            );
        }
        while tx_order_start <= last_order {
            blocks.push(BlockRange {
                block_number,
                tx_order_start,
                tx_order_end,
            });
            tx_order_start = tx_order_end + 1;
            tx_order_end = min(
                tx_order_start + MAX_TXS_PER_BLOCK_IN_FIX as u64 - 1,
                last_order,
            );
            block_number += 1;
        }
        Ok(blocks)
    }

    fn append_submitting_blocks(&self, mut ranges: Vec<BlockRange>) -> anyhow::Result<()> {
        if ranges.is_empty() {
            return Ok(());
        }

        ranges.sort_by(|a, b| a.block_number.cmp(&b.block_number));
        let last_block_number = ranges.last().unwrap().block_number;

        let inner_store = self.block_submit_state_store.get_store().store();
        let mut cf_batches: Vec<WriteBatchCF> = Vec::new();

        let state_batch = WriteBatchCF::new_with_rows(
            ranges
                .iter()
                .map(|range| {
                    let key = to_bytes(&range.block_number).unwrap();
                    let value = to_bytes(&BlockSubmitState::new(
                        range.block_number,
                        range.tx_order_start,
                        range.tx_order_end,
                    ))
                    .unwrap();
                    (key, WriteOp::Value(value))
                })
                .collect(),
            DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME.to_string(),
        );
        cf_batches.push(state_batch);

        let last_block_batch = WriteBatchCF {
            batch: WriteBatch::new_with_rows(vec![(
                to_bytes(LAST_BLOCK_NUMBER_KEY).unwrap(),
                WriteOp::Value(to_bytes(&last_block_number).unwrap()),
            )]),
            cf_name: DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME.to_string(),
        };
        cf_batches.push(last_block_batch);

        inner_store.write_cf_batch(cf_batches, true)?;
        Ok(())
    }

    // check every blocks' tx_order_start and tx_order_end: [0, last_block_number]:
    // 1. ensure tx_order_start <= tx_order_end for each block
    // 2. ensure block_i's tx_order_end +1 = block_i+1's tx_order_start
    pub(crate) fn try_find_first_illegal(
        &self,
        last_block_number: u128,
        last_order: u64,
    ) -> anyhow::Result<Option<u128>> {
        // At least has one block (invoking precondition)
        let block_0_range = self.get_block_state(0)?.block_range;
        if !block_0_range.is_legal(last_order) {
            return Ok(Some(0));
        }

        let mut last_block_tx_order_end = block_0_range.tx_order_end;
        let mut first_illegal: Option<u128> = None;

        for i in 1..=last_block_number {
            let block_state = self.get_block_state(i)?;
            let block_range = BlockRange {
                block_number: i,
                tx_order_start: block_state.block_range.tx_order_start,
                tx_order_end: block_state.block_range.tx_order_end,
            };
            if !block_range.is_legal(last_order) {
                first_illegal = Some(i);
                break;
            }
            if block_range.tx_order_start != last_block_tx_order_end + 1 {
                first_illegal = Some(i);
                break;
            } else {
                last_block_tx_order_end = block_range.tx_order_end;
            }
        }
        Ok(first_illegal)
    }

    // find first illegal block(n), then rollback to n-1
    // if n == 0, then remove all blocks
    //
    // tx orders issues may be caused by:
    // break changes causes inconsistent tx orders (after v0.7.6 is stable)
    pub(crate) fn try_repair_orders(&self, last_order: u64) -> anyhow::Result<(usize, usize)> {
        let last_block_number = self.get_last_block_number()?;
        let mut issues = 0;
        match last_block_number {
            None => Ok((0, 0)),
            Some(last_block_number) => {
                let first_illegal = self.try_find_first_illegal(last_block_number, last_order)?;
                match first_illegal {
                    None => Ok((0, 0)),
                    Some(first_illegal) => {
                        let mut remove_blocks = Vec::new();
                        for block_number in first_illegal..=last_block_number {
                            remove_blocks.push(block_number);
                        }
                        issues += remove_blocks.len();
                        self.inner_rollback(remove_blocks)?;
                        Ok((issues, issues))
                    }
                }
            }
        }
    }

    pub(crate) fn try_repair_blocks(
        &self,
        last_order: u64,
        mut issues: usize,
        mut fixed: usize,
    ) -> anyhow::Result<(usize, usize)> {
        let last_block_number = self.get_last_block_number()?;
        match last_block_number {
            Some(last_block_number) => {
                let last_block_state = self.get_block_state(last_block_number)?;
                let last_block_order_end = last_block_state.block_range.tx_order_end;

                match last_order.cmp(&last_block_order_end) {
                    Ordering::Greater => {
                        let append_count =
                            self.append_block_by_repair(Some(last_block_number), last_order)?;
                        issues += append_count;
                        fixed += append_count;
                        Ok((issues, fixed))
                    }
                    Ordering::Less => {
                        let remove_blocks = self.generate_remove_blocks_after_order(
                            Some(last_block_number),
                            last_order,
                        )?;
                        issues += remove_blocks.len();
                        fixed += remove_blocks.len();
                        self.inner_rollback(remove_blocks)?;
                        self.try_repair_blocks(last_order, issues, fixed)
                    }
                    Ordering::Equal => Ok((issues, fixed)),
                }
            }
            None => {
                if last_order == 0 {
                    Ok((issues, fixed))
                } else {
                    let append_count = self.append_block_by_repair(None, last_order)?;
                    issues += append_count;
                    fixed += append_count;
                    Ok((issues, fixed))
                }
            }
        }
    }

    // append won't be invoked frequently, so the extra cost of checking is acceptable
    fn check_append(
        &self,
        last_block_number: Option<u128>,
        tx_order_start: u64,
        tx_order_end: u64,
    ) -> anyhow::Result<()> {
        if tx_order_end < tx_order_start {
            return Err(anyhow::anyhow!(
                "tx_order_end must >= tx_order_start, got {} < {}",
                tx_order_end,
                tx_order_start
            ));
        }
        match last_block_number {
            None => Ok(()),
            Some(block_number) => {
                let last_block_state = self.get_block_state(block_number)?;
                if last_block_state.block_range.tx_order_end + 1 != tx_order_start {
                    return Err(anyhow::anyhow!(
                        "tx_order_start must be last_block_number's tx_order_end + 1, last_tx_order_end {}, tx_order_start {}",
                        last_block_state.block_range.tx_order_end,
                        tx_order_start
                    ));
                }
                Ok(())
            }
        }
    }
}

impl DAMetaStore for DAMetaDBStore {
    fn try_repair_da_meta(
        &self,
        last_order: u64,
        thorough: bool,
    ) -> anyhow::Result<(usize, usize)> {
        let mut issues = 0;
        let mut fixed = 0;
        if thorough {
            let (order_issues, order_fixed) = self.try_repair_orders(last_order)?;
            issues += order_issues;
            fixed += order_fixed;
        }
        (issues, fixed) = self.try_repair_blocks(last_order, issues, fixed)?;

        Ok((issues, fixed))
    }

    fn append_submitting_block(
        &self,
        tx_order_start: u64,
        tx_order_end: u64,
    ) -> anyhow::Result<u128> {
        let last_block_number = self.get_last_block_number()?;

        self.check_append(last_block_number, tx_order_start, tx_order_end)?;

        let inner_store = self.block_submit_state_store.store.store();

        let block_number = match last_block_number {
            Some(last_block_number) => last_block_number + 1,
            None => 0,
        };
        let submit_state = BlockSubmitState::new(block_number, tx_order_start, tx_order_end);
        let block_number_bytes = to_bytes(&block_number)?;
        let submit_state_bytes = to_bytes(&submit_state)?;
        let last_block_number_key_bytes = to_bytes(LAST_BLOCK_NUMBER_KEY)?;
        let mut write_batch = WriteBatch::new();
        write_batch.put(block_number_bytes.clone(), submit_state_bytes)?;
        write_batch.put(last_block_number_key_bytes, block_number_bytes.clone())?;
        inner_store.write_batch_across_cfs(
            vec![
                DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME,
                DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME,
            ],
            write_batch,
            // sync write for:
            // db may collapse after:
            // 1. the block has been submitted
            // 2. proposer has added the block
            // after recovery, the range of block may change, scc & DA will be inconsistent
            true,
        )?;

        Ok(block_number)
    }

    fn get_submitting_blocks(
        &self,
        start_block: u128,
        exp_count: Option<usize>,
    ) -> anyhow::Result<Vec<BlockRange>> {
        let exp_count = exp_count.unwrap_or(SUBMITTING_BLOCKS_PAGE_SIZE);
        // try to get exp_count unsubmitted blocks
        // TODO use multi-get to get unsubmitted blocks
        let mut blocks = Vec::with_capacity(exp_count);
        let mut found = 0;
        let mut block_number = start_block;
        while found < exp_count {
            let state = self.block_submit_state_store.kv_get(block_number)?;
            if let Some(state) = state {
                if !state.done {
                    blocks.push(BlockRange {
                        block_number,
                        tx_order_start: state.block_range.tx_order_start,
                        tx_order_end: state.block_range.tx_order_end,
                    });
                    found += 1;
                }
            } else {
                break; // no more blocks
            }
            block_number += 1;
        }

        Ok(blocks)
    }

    // TODO combine set_submitting_block_done and set_background_submit_block_cursor: no more user-facing submit block
    fn set_submitting_block_done(
        &self,
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        batch_hash: H256,
    ) -> anyhow::Result<()> {
        self.block_submit_state_store.kv_put(
            block_number,
            BlockSubmitState::new_done(block_number, tx_order_start, tx_order_end, batch_hash),
        )
    }

    fn set_background_submit_block_cursor(&self, cursor: u128) -> anyhow::Result<()> {
        self.block_cursor_store
            .kv_put(BACKGROUND_SUBMIT_BLOCK_CURSOR_KEY.to_string(), cursor)
    }

    fn get_background_submit_block_cursor(&self) -> anyhow::Result<Option<u128>> {
        self.block_cursor_store
            .kv_get(BACKGROUND_SUBMIT_BLOCK_CURSOR_KEY.to_string())
    }

    fn get_last_block_number(&self) -> anyhow::Result<Option<u128>> {
        self.block_cursor_store
            .kv_get(LAST_BLOCK_NUMBER_KEY.to_string())
    }

    fn get_block_state(&self, block_number: u128) -> anyhow::Result<BlockSubmitState> {
        self.get_block_state_opt(block_number)?.ok_or_else(|| {
            anyhow::anyhow!("block submit state not found for block: {}", block_number)
        })
    }

    fn try_get_block_state(&self, block_number: u128) -> anyhow::Result<Option<BlockSubmitState>> {
        self.get_block_state_opt(block_number)
    }
}
