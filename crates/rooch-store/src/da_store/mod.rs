// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME, DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME};
use moveos_common::utils::to_bytes;
use raw_store::rocks::batch::WriteBatch;
use raw_store::traits::DBStore;
use raw_store::{derive_store, CodecKVStore, CodecWriteBatch};
use rooch_types::da::batch::{BlockRange, BlockSubmitState};
use std::cmp::min;

pub const SUBMITTING_BLOCKS_PAGE_SIZE: usize = 1024;
pub const MAX_TXS_PER_BLOCK_IN_FIX: usize = 8192; // avoid OOM when fix submitting blocks after collapse

pub const BACKGROUND_SUBMIT_BLOCK_CURSOR_KEY: &str = "background_submit_block_cursor";
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
    /// Get submitting blocks from start_block(inclusive) with expected count until the end of submitting blocks
    fn get_submitting_blocks(
        &self,
        start_block: u128,
        exp_count: Option<usize>,
    ) -> anyhow::Result<Vec<BlockRange>>;
    fn append_submitting_block(
        &self,
        last_block_number: Option<u128>,
        tx_order_start: u64,
        tx_order_end: u64,
    ) -> anyhow::Result<u128>;
    fn set_submitting_block_done(
        &self,
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
    ) -> anyhow::Result<()>;

    fn get_background_submit_block_cursor(&self) -> anyhow::Result<Option<u128>>;
    fn set_background_submit_block_cursor(&self, cursor: u128) -> anyhow::Result<()>;

    fn get_last_block_number(&self) -> anyhow::Result<Option<u128>>;

    // try to fix submitting blocks by last tx order(if not None) at starting,
    // only could be called after try_fix_last_block_number(which has been invoked in new)
    fn catchup_submitting_blocks(&self, last_order: Option<u64>) -> anyhow::Result<()>;
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
        store.try_fix_last_block_number()?;
        Ok(store)
    }

    // update last block and update submitting blocks are not in the same transaction
    // so there may be a case that the submitting blocks is updated but last block is not updated
    pub(crate) fn try_fix_last_block_number(&self) -> anyhow::Result<()> {
        let last_block_number = self.get_last_block_number()?;
        if let Some(last_block_number) = last_block_number {
            let blocks = self.get_submitting_blocks(last_block_number + 1, None)?;
            if let Some(block) = blocks.last() {
                self.set_last_block_number(block.block_number)?;
            }
        }
        Ok(())
    }

    pub(crate) fn set_last_block_number(&self, block_number: u128) -> anyhow::Result<()> {
        self.block_cursor_store
            .put_sync(LAST_BLOCK_NUMBER_KEY.to_string(), block_number)
    }

    fn add_submitting_blocks(&self, ranges: Vec<BlockRange>) -> anyhow::Result<()> {
        if ranges.is_empty() {
            return Ok(());
        }

        let last_block_number = ranges.last().unwrap().block_number;
        let kvs: Vec<(u128, BlockSubmitState)> = ranges
            .into_iter()
            .map(|range| {
                (
                    range.block_number,
                    BlockSubmitState::new(
                        range.block_number,
                        range.tx_order_start,
                        range.tx_order_end,
                    ),
                )
            })
            .collect();
        let batch = CodecWriteBatch::new_puts(kvs);
        self.block_submit_state_store.write_batch_sync(batch)?;

        self.set_last_block_number(last_block_number)?;
        Ok(())
    }

    pub(crate) fn calc_needed_block_for_fix_submitting(
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
            let last_block_state = self
                .block_submit_state_store
                .kv_get(last_block_number)?
                .unwrap_or_else(|| {
                    panic!(
                        "submitting block not found for existed last block: {}",
                        last_block_number
                    )
                });
            let last_range = last_block_state.block_range;
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

    // try to fix submitting blocks by last tx order(if not None) at starting,
    // only could be called after try_fix_last_block_number
    fn try_fix_submitting_blocks(&self, last_order: Option<u64>) -> anyhow::Result<()> {
        if let Some(last_order) = last_order {
            if last_order == 0 {
                // only has genesis_tx
                return Ok(());
            }
            let last_block_number = self.get_last_block_number()?;
            let ranges =
                self.calc_needed_block_for_fix_submitting(last_block_number, last_order)?;
            self.add_submitting_blocks(ranges)?;
        }
        Ok(())
    }
}

impl DAMetaStore for DAMetaDBStore {
    fn get_submitting_blocks(
        &self,
        start_block: u128,
        exp_count: Option<usize>,
    ) -> anyhow::Result<Vec<BlockRange>> {
        let exp_count = exp_count.unwrap_or(SUBMITTING_BLOCKS_PAGE_SIZE);
        // try to get exp_count unsubmitted blocks
        let mut blocks = Vec::with_capacity(exp_count);
        let mut done_count = 0;
        let mut block_number = start_block;
        while done_count < exp_count {
            let state = self.block_submit_state_store.kv_get(block_number)?;
            if let Some(state) = state {
                if !state.done {
                    blocks.push(BlockRange {
                        block_number,
                        tx_order_start: state.block_range.tx_order_start,
                        tx_order_end: state.block_range.tx_order_end,
                    });
                    done_count += 1;
                }
            } else {
                break;
            }
            block_number += 1;
        }

        Ok(blocks)
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

        inner_store.write_batch_sync_across_cfs(
            vec![
                DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME,
                DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME,
            ],
            write_batch,
        )?;

        Ok(block_number)
    }

    fn set_submitting_block_done(
        &self,
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
    ) -> anyhow::Result<()> {
        self.block_submit_state_store.kv_put(
            block_number,
            BlockSubmitState::new_done(block_number, tx_order_start, tx_order_end),
        )
    }

    fn get_background_submit_block_cursor(&self) -> anyhow::Result<Option<u128>> {
        self.block_cursor_store
            .kv_get(BACKGROUND_SUBMIT_BLOCK_CURSOR_KEY.to_string())
    }

    fn set_background_submit_block_cursor(&self, cursor: u128) -> anyhow::Result<()> {
        self.block_cursor_store
            .kv_put(BACKGROUND_SUBMIT_BLOCK_CURSOR_KEY.to_string(), cursor)
    }

    fn get_last_block_number(&self) -> anyhow::Result<Option<u128>> {
        self.block_cursor_store
            .kv_get(LAST_BLOCK_NUMBER_KEY.to_string())
    }

    fn catchup_submitting_blocks(&self, last_order: Option<u64>) -> anyhow::Result<()> {
        self.try_fix_submitting_blocks(last_order)
    }
}
