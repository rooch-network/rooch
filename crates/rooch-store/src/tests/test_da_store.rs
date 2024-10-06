// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::da_store::{DAMetaStore, MAX_TXS_PER_BLOCK_IN_FIX};
use crate::RoochStore;
use std::cmp::min;

#[tokio::test]
async fn test_append_submitting_blocks() {
    let (rooch_store, _) = RoochStore::mock_rooch_store().unwrap();
    let da_meta_store = rooch_store.get_da_meta_store();

    da_meta_store.append_submitting_block(None, 0, 6).unwrap();

    let last_block_number = 0;
    let tx_order_start = 7;
    let tx_order_end = 7;
    da_meta_store
        .append_submitting_block(Some(last_block_number), tx_order_start, tx_order_end)
        .unwrap();

    let submitting_blocks = da_meta_store.get_submitting_blocks(0, None).unwrap();
    assert_eq!(submitting_blocks.len(), 2);
    assert_eq!(submitting_blocks[0].block_number, 0);
    assert_eq!(submitting_blocks[0].tx_order_start, 0);
    assert_eq!(submitting_blocks[0].tx_order_end, 6);
    assert_eq!(submitting_blocks[1].block_number, 1);
    assert_eq!(submitting_blocks[1].tx_order_start, 7);
    assert_eq!(submitting_blocks[1].tx_order_end, 7);

    let submitting_blocks = da_meta_store.get_submitting_blocks(1, None).unwrap();
    assert_eq!(submitting_blocks.len(), 1);
    assert_eq!(submitting_blocks[0].block_number, 1);
    assert_eq!(submitting_blocks[0].tx_order_start, 7);
    assert_eq!(submitting_blocks[0].tx_order_end, 7);
}

#[tokio::test]
async fn test_try_fix_last_block_number() {
    let (rooch_store, _) = RoochStore::mock_rooch_store().unwrap();
    let da_meta_store = rooch_store.get_da_meta_store();

    let tx_order_start = 7;
    let tx_order_end = 7;
    let last_block_number = da_meta_store
        .append_submitting_block(None, tx_order_start, tx_order_end)
        .unwrap();
    da_meta_store
        .append_submitting_block(Some(last_block_number), tx_order_start, tx_order_end)
        .unwrap();
    da_meta_store.set_last_block_number(0).unwrap();
    assert_eq!(da_meta_store.get_last_block_number().unwrap().unwrap(), 0);
    let submitting_blocks = da_meta_store.get_submitting_blocks(0, None).unwrap();
    assert_eq!(submitting_blocks.len(), 2);
    assert_eq!(submitting_blocks[1].block_number, 1);
    assert_eq!(submitting_blocks[1].tx_order_start, 7);
    assert_eq!(submitting_blocks[1].tx_order_end, 7);
    da_meta_store.try_fix_last_block_number().unwrap();
    assert_eq!(da_meta_store.get_last_block_number().unwrap().unwrap(), 1);
}

#[tokio::test]
async fn test_calc_needed_block_for_fix_submitting() {
    let (rooch_store, _) = RoochStore::mock_rooch_store().unwrap();
    let da_meta_store = rooch_store.get_da_meta_store();

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(None, 10)
        .unwrap();
    assert_eq!(needed_blocks.len(), 1);
    assert_eq!(needed_blocks[0].block_number, 0);
    assert_eq!(needed_blocks[0].tx_order_start, 0);
    assert_eq!(needed_blocks[0].tx_order_end, 10);

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(None, 0)
        .unwrap();
    assert_eq!(needed_blocks.len(), 1);
    assert_eq!(needed_blocks[0].block_number, 0);
    assert_eq!(needed_blocks[0].tx_order_start, 0);
    assert_eq!(needed_blocks[0].tx_order_end, 0);

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(None, MAX_TXS_PER_BLOCK_IN_FIX as u64 - 1)
        .unwrap();
    assert_eq!(needed_blocks.len(), 1);
    assert_eq!(needed_blocks[0].block_number, 0);
    assert_eq!(needed_blocks[0].tx_order_start, 0);
    assert_eq!(
        needed_blocks[0].tx_order_end,
        MAX_TXS_PER_BLOCK_IN_FIX as u64 - 1
    );

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(None, MAX_TXS_PER_BLOCK_IN_FIX as u64)
        .unwrap();
    assert_eq!(needed_blocks.len(), 2);
    assert_eq!(needed_blocks[1].block_number, 1);
    assert_eq!(
        needed_blocks[1].tx_order_start,
        MAX_TXS_PER_BLOCK_IN_FIX as u64
    );
    assert_eq!(
        needed_blocks[1].tx_order_end,
        MAX_TXS_PER_BLOCK_IN_FIX as u64
    );

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(None, MAX_TXS_PER_BLOCK_IN_FIX as u64 + 1)
        .unwrap();
    assert_eq!(needed_blocks.len(), 2);
    assert_eq!(needed_blocks[1].block_number, 1);
    assert_eq!(
        needed_blocks[1].tx_order_start,
        MAX_TXS_PER_BLOCK_IN_FIX as u64
    );
    assert_eq!(
        needed_blocks[1].tx_order_end,
        MAX_TXS_PER_BLOCK_IN_FIX as u64 + 1
    );

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(None, 9 + 2 * MAX_TXS_PER_BLOCK_IN_FIX as u64)
        .unwrap();
    assert_eq!(needed_blocks.len(), 3);
    assert_eq!(needed_blocks[2].block_number, 2);
    for i in 0..2 {
        assert_eq!(
            needed_blocks[i].tx_order_start,
            i as u64 * MAX_TXS_PER_BLOCK_IN_FIX as u64
        );
        assert_eq!(
            needed_blocks[i].tx_order_end,
            min(
                (i + 1) as u64 * MAX_TXS_PER_BLOCK_IN_FIX as u64 - 1,
                9 + 2 * MAX_TXS_PER_BLOCK_IN_FIX as u64
            )
        );
    }

    let tx_order_start = 7;
    let tx_order_end = 7;
    let last_block_number = da_meta_store
        .append_submitting_block(None, tx_order_start, tx_order_end)
        .unwrap();
    da_meta_store
        .append_submitting_block(Some(last_block_number), tx_order_start, tx_order_end)
        .unwrap();
    da_meta_store.set_last_block_number(0).unwrap();
    assert_eq!(da_meta_store.get_last_block_number().unwrap().unwrap(), 0);
    let submitting_blocks = da_meta_store.get_submitting_blocks(0, None).unwrap();
    assert_eq!(submitting_blocks.len(), 2);
    assert_eq!(submitting_blocks[1].block_number, 1);
    assert_eq!(submitting_blocks[1].tx_order_start, 7);
    assert_eq!(submitting_blocks[1].tx_order_end, 7);
    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(Some(1), 10)
        .unwrap();
    assert_eq!(needed_blocks.len(), 1);
    assert_eq!(needed_blocks[0].block_number, 2);
    assert_eq!(needed_blocks[0].tx_order_start, 8);
    assert_eq!(needed_blocks[0].tx_order_end, 10);

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(Some(1), 7)
        .unwrap();
    assert_eq!(needed_blocks.len(), 0);

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(Some(1), 8)
        .unwrap();
    assert_eq!(needed_blocks.len(), 1);
    assert_eq!(needed_blocks[0].block_number, 2);
    assert_eq!(needed_blocks[0].tx_order_start, 8);
    assert_eq!(needed_blocks[0].tx_order_end, 8);

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(Some(1), 7 + MAX_TXS_PER_BLOCK_IN_FIX as u64)
        .unwrap();
    assert_eq!(needed_blocks.len(), 1);
    assert_eq!(needed_blocks[0].block_number, 2);
    assert_eq!(needed_blocks[0].tx_order_start, 8);
    assert_eq!(
        needed_blocks[0].tx_order_end,
        7 + MAX_TXS_PER_BLOCK_IN_FIX as u64
    );

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(Some(1), 8 + MAX_TXS_PER_BLOCK_IN_FIX as u64)
        .unwrap();
    assert_eq!(needed_blocks.len(), 2);
    assert_eq!(needed_blocks[1].block_number, 3);
    assert_eq!(
        needed_blocks[1].tx_order_start,
        8 + MAX_TXS_PER_BLOCK_IN_FIX as u64
    );
    assert_eq!(
        needed_blocks[1].tx_order_end,
        8 + MAX_TXS_PER_BLOCK_IN_FIX as u64
    );

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(Some(1), 9 + MAX_TXS_PER_BLOCK_IN_FIX as u64)
        .unwrap();
    assert_eq!(needed_blocks.len(), 2);
    assert_eq!(needed_blocks[1].block_number, 3);
    assert_eq!(
        needed_blocks[1].tx_order_start,
        8 + MAX_TXS_PER_BLOCK_IN_FIX as u64
    );
    assert_eq!(
        needed_blocks[1].tx_order_end,
        9 + MAX_TXS_PER_BLOCK_IN_FIX as u64
    );

    let needed_blocks = da_meta_store
        .calc_needed_block_for_fix_submitting(Some(1), 9 + 2 * MAX_TXS_PER_BLOCK_IN_FIX as u64)
        .unwrap();
    assert_eq!(needed_blocks.len(), 3);
    assert_eq!(needed_blocks[2].block_number, 4);
    for i in 1..3 {
        assert_eq!(
            needed_blocks[i].tx_order_start,
            8 + i as u64 * MAX_TXS_PER_BLOCK_IN_FIX as u64
        );
        assert_eq!(
            needed_blocks[i].tx_order_end,
            min(
                8 + (i + 1) as u64 * MAX_TXS_PER_BLOCK_IN_FIX as u64 - 1,
                9 + 2 * MAX_TXS_PER_BLOCK_IN_FIX as u64
            )
        );
    }
}
