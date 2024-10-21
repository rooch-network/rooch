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
    // try to repair by last tx order at starting:
    // 1. last_tx_order is ahead of last_block_number's tx_order_end: appending submitting blocks until last_order(inclusive)
    // 2. last_tx_order is behind last_block_number's tx_order_end: remove blocks which tx_order_end > last_order
    //   (caused by offline rooch-db rollback/revert cmd):
    //   a. remove blocks from last_block_number to the block which tx_order_end is ahead of last_tx_order
    //   b. update last_block_number to the block which tx_order_end is behind of last_order
    //   c. remove background_submit_block_cursor directly, since we could catch up with the last order by background submitter
    // after repair with condition2, we may need to repair with condition1 for the last block(it will be done automatically)
    fn try_repair(&self, last_order: u64) -> anyhow::Result<()>;

    // append new submitting block with tx_order_start and tx_order_end
    // warning: not thread safe
    fn append_submitting_block(
        &self,
        last_block_number: Option<u128>,
        tx_order_start: u64,
        tx_order_end: u64,
    ) -> anyhow::Result<u128>;
    // get submitting blocks(block is not submitted) from start_block(inclusive) with expected count until the end of submitting blocks
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
    ) -> anyhow::Result<()> {
        let block_ranges = self.generate_append_blocks(last_block_number, last_order)?;
        self.append_submitting_block(block_ranges)
    }

    fn inner_rollback(
        &self,
        last_block_number: Option<u128>,
        last_order: u64,
    ) -> anyhow::Result<()> {
        let mut remove_blocks = self.generate_remove_blocks(last_block_number, last_order)?;
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

        inner_store.write_cf_batch(cf_batches, true)
    }

    // generate the block ranges to remove blocks
    pub(crate) fn generate_remove_blocks(
        &self,
        last_block_number: Option<u128>,
        last_order: u64,
    ) -> anyhow::Result<Vec<u128>> {
        let mut blocks = Vec::new();

        if let Some(mut block_number) = last_block_number {
            loop {
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

    fn append_submitting_block(&self, mut ranges: Vec<BlockRange>) -> anyhow::Result<()> {
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

    pub(crate) fn get_block_state(&self, block_number: u128) -> anyhow::Result<BlockSubmitState> {
        self.block_submit_state_store
            .kv_get(block_number)?
            .ok_or_else(|| {
                anyhow::anyhow!("block submit state not found for block: {}", block_number)
            })
    }
}

impl DAMetaStore for DAMetaDBStore {
    fn try_repair(&self, last_order: u64) -> anyhow::Result<()> {
        let last_block_number = self.get_last_block_number()?;
        match last_block_number {
            Some(last_block_number) => {
                let last_block_state = self.get_block_state(last_block_number)?;
                let last_block_order_end = last_block_state.block_range.tx_order_end;

                match last_order.cmp(&last_block_order_end) {
                    Ordering::Greater => {
                        self.append_block_by_repair(Some(last_block_number), last_order)
                    }
                    Ordering::Less => {
                        self.inner_rollback(Some(last_block_number), last_order)?;
                        self.try_repair(last_order)
                    }
                    Ordering::Equal => Ok(()),
                }
            }
            None => {
                if last_order == 0 {
                    Ok(())
                } else {
                    self.append_block_by_repair(None, last_order)
                }
            }
        }
    }

    fn append_submitting_block(
        &self,
        last_block_number: Option<u128>,
        tx_order_start: u64,
        tx_order_end: u64,
    ) -> anyhow::Result<u128> {
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
}
