// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::da_store::{DAMetaDBStore, DAMetaStore, MAX_TXS_PER_BLOCK_IN_FIX};
use crate::RoochStore;
use rooch_types::da::batch::BlockRange;

#[tokio::test]
async fn get_submitting_blocks() {
    let (rooch_store, _) = RoochStore::mock_rooch_store().unwrap();
    let da_meta_store = rooch_store.get_da_meta_store();

    da_meta_store.append_submitting_block(1, 6).unwrap();

    let tx_order_start = 7;
    let tx_order_end = 7;
    da_meta_store
        .append_submitting_block(tx_order_start, tx_order_end)
        .unwrap();

    let submitting_blocks = da_meta_store.get_submitting_blocks(0, None).unwrap();
    assert_eq!(submitting_blocks.len(), 2);
    assert_eq!(submitting_blocks[0].block_number, 0);
    assert_eq!(submitting_blocks[0].tx_order_start, 1);
    assert_eq!(submitting_blocks[0].tx_order_end, 6);
    assert_eq!(submitting_blocks[1].block_number, 1);
    assert_eq!(submitting_blocks[1].tx_order_start, 7);
    assert_eq!(submitting_blocks[1].tx_order_end, 7);

    let submitting_blocks = da_meta_store.get_submitting_blocks(0, Some(2)).unwrap();
    assert_eq!(submitting_blocks.len(), 2);
    assert_eq!(submitting_blocks[0].block_number, 0);
    assert_eq!(submitting_blocks[0].tx_order_start, 1);
    assert_eq!(submitting_blocks[0].tx_order_end, 6);
    assert_eq!(submitting_blocks[1].block_number, 1);
    assert_eq!(submitting_blocks[1].tx_order_start, 7);
    assert_eq!(submitting_blocks[1].tx_order_end, 7);

    let submitting_blocks = da_meta_store.get_submitting_blocks(1, None).unwrap();
    assert_eq!(submitting_blocks.len(), 1);
    assert_eq!(submitting_blocks[0].block_number, 1);
    assert_eq!(submitting_blocks[0].tx_order_start, 7);
    assert_eq!(submitting_blocks[0].tx_order_end, 7);

    let submitting_blocks = da_meta_store.get_submitting_blocks(1, Some(1)).unwrap();
    assert_eq!(submitting_blocks.len(), 1);
    assert_eq!(submitting_blocks[0].block_number, 1);
    assert_eq!(submitting_blocks[0].tx_order_start, 7);
    assert_eq!(submitting_blocks[0].tx_order_end, 7);
}

#[tokio::test]
async fn generate_remove_blocks() {
    let (rooch_store, _) = RoochStore::mock_rooch_store().unwrap();
    let da_meta_store = rooch_store.get_da_meta_store();

    da_meta_store.append_submitting_block(1, 6).unwrap();
    da_meta_store.append_submitting_block(7, 7).unwrap();
    da_meta_store.append_submitting_block(8, 1024).unwrap();

    fn check_remove_blocks(
        case: u64,
        store: &DAMetaDBStore,
        last_block_opt: Option<u128>,
        last_order: u64,
        expected_len: usize,
        expected_val: Option<Vec<u128>>,
    ) {
        let mut remove_blocks = store
            .generate_remove_blocks_after_order(last_block_opt, last_order)
            .unwrap();
        remove_blocks.sort();
        assert_eq!(remove_blocks.len(), expected_len, "test case {}", case);
        if let Some(expected) = expected_val {
            assert_eq!(remove_blocks, expected, "test case {}", case);
        }
    }

    check_remove_blocks(0, da_meta_store, None, 0, 0, None);
    check_remove_blocks(1, da_meta_store, Some(0), 1, 1, Some(vec![0]));
    check_remove_blocks(3, da_meta_store, Some(0), 6, 0, None);
    check_remove_blocks(4, da_meta_store, Some(1), 6, 1, Some(vec![1]));
    check_remove_blocks(5, da_meta_store, Some(1), 7, 0, None);
    check_remove_blocks(6, da_meta_store, Some(2), 0, 3, Some(vec![0, 1, 2]));
    check_remove_blocks(7, da_meta_store, Some(2), 1, 3, Some(vec![0, 1, 2]));
    check_remove_blocks(8, da_meta_store, Some(2), 2, 3, Some(vec![0, 1, 2]));
    check_remove_blocks(9, da_meta_store, Some(2), 6, 2, Some(vec![1, 2]));
    check_remove_blocks(10, da_meta_store, Some(2), 7, 1, Some(vec![2]));
    check_remove_blocks(11, da_meta_store, Some(2), 8, 1, Some(vec![2]));
    check_remove_blocks(12, da_meta_store, Some(2), 1023, 1, Some(vec![2]));
    check_remove_blocks(13, da_meta_store, Some(2), 1024, 0, None);
}

#[tokio::test]
async fn catch_up_last_tx_order() {
    let (rooch_store, _) = RoochStore::mock_rooch_store().unwrap();

    run_catch_up_last_tx_order_case(0, None, 0, rooch_store.clone(), None);
    run_catch_up_last_tx_order_case(1, None, 1, rooch_store.clone(), Some(0));
    run_catch_up_last_tx_order_case(2, None, 10, rooch_store.clone(), Some(0));
    run_catch_up_last_tx_order_case(
        3,
        None,
        MAX_TXS_PER_BLOCK_IN_FIX as u64,
        rooch_store.clone(),
        Some(0),
    );
    run_catch_up_last_tx_order_case(
        4,
        None,
        MAX_TXS_PER_BLOCK_IN_FIX as u64 + 1,
        rooch_store.clone(),
        Some(0),
    );
    run_catch_up_last_tx_order_case(
        5,
        None,
        MAX_TXS_PER_BLOCK_IN_FIX as u64 + 2,
        rooch_store.clone(),
        Some(0),
    );
    run_catch_up_last_tx_order_case(
        6,
        None,
        9 + 2 * MAX_TXS_PER_BLOCK_IN_FIX as u64,
        rooch_store.clone(),
        Some(0),
    );

    let tx_order_end = 3 * MAX_TXS_PER_BLOCK_IN_FIX as u64 + 1;
    let last_block_number = rooch_store.append_submitting_block(1, 3).unwrap();
    assert_eq!(last_block_number, 0);
    let last_block_number = rooch_store
        .append_submitting_block(4, tx_order_end)
        .unwrap();
    assert_eq!(last_block_number, 1);

    for i in 1..2 * MAX_TXS_PER_BLOCK_IN_FIX + 2 {
        run_catch_up_last_tx_order_case(
            (6 + i) as u64,
            Some(last_block_number),
            tx_order_end + i as u64,
            rooch_store.clone(),
            Some(last_block_number + 1),
        );
    }
}

fn run_catch_up_last_tx_order_case(
    case: u64,
    last_block_number: Option<u128>,
    last_order: u64,
    rooch_store: RoochStore,
    exp_first_block_number: Option<u128>,
) {
    let da_meta_store = rooch_store.get_da_meta_store();

    let block_ranges = da_meta_store
        .generate_append_blocks(last_block_number, last_order)
        .unwrap();

    let origin_tx_order_end = if let Some(last_block_number) = last_block_number {
        let state = da_meta_store.get_block_state(last_block_number).unwrap();
        Some(state.block_range.tx_order_end)
    } else {
        None
    };

    check_block_ranges(
        case,
        block_ranges,
        exp_first_block_number,
        last_order,
        origin_tx_order_end,
    );
}

fn check_block_ranges(
    test_case: u64,
    block_ranges: Vec<BlockRange>,
    exp_first_block_number: Option<u128>,
    last_order: u64,
    origin_tx_order_end: Option<u64>,
) {
    if exp_first_block_number.is_none() {
        assert_eq!(
            block_ranges.len(),
            0,
            "Test case {}: expected no block ranges, but got {:#?}",
            test_case,
            block_ranges,
        );
        return;
    }
    assert!(
        !block_ranges.is_empty(),
        "Test case {}: expected some block ranges, but got none",
        test_case,
    );

    let exp_first_block_number = exp_first_block_number.unwrap();
    let act_first_block_number = block_ranges.first().unwrap().block_number;
    assert_eq!(
        act_first_block_number, exp_first_block_number,
        "Test case {}: first block number mismatch, expected {}, got {}",
        test_case, exp_first_block_number, act_first_block_number,
    );

    let tx_start = block_ranges.first().unwrap().tx_order_start;
    assert!(
        tx_start > 0,
        "Test case {}: first order mismatch, expected > 0, got {}",
        test_case,
        tx_start,
    );

    let tx_end = block_ranges.last().unwrap().tx_order_end;
    assert_eq!(
        tx_end, last_order,
        "Test case {}: last order mismatch, expected {}, got {}",
        test_case, last_order, tx_end,
    );

    for block_range in &block_ranges {
        let txs = block_range.tx_order_end - block_range.tx_order_start + 1;
        assert!(
            txs <= MAX_TXS_PER_BLOCK_IN_FIX as u64,
            "Test case {}: too many txs in block range {:#?}, max allowed {}",
            test_case,
            block_range,
            MAX_TXS_PER_BLOCK_IN_FIX
        );
        assert!(
            block_range.tx_order_start <= block_range.tx_order_end,
            "Test case {}: tx_order_start > tx_order_end in block range {:#?}",
            test_case,
            block_range,
        );
    }

    for i in 0..block_ranges.len() - 1 {
        let tx_order_end = block_ranges[i].tx_order_end;
        let tx_order_start = block_ranges[i + 1].tx_order_start;
        assert_eq!(
            tx_order_end + 1,
            tx_order_start,
            "Test case {}: tx_order continuity issue between blocks {:#?} and {:#?}",
            test_case,
            block_ranges[i],
            block_ranges[i + 1]
        );

        let block_number = block_ranges[i].block_number;
        let next_block_number = block_ranges[i + 1].block_number;
        assert_eq!(
            block_number + 1,
            next_block_number,
            "Test case {}: block number continuity issue between blocks {:#?} and {:#?}",
            test_case,
            block_ranges[i],
            block_ranges[i + 1]
        );
    }

    let total_txs: u64 = block_ranges
        .iter()
        .map(|block_range| block_range.tx_order_end - block_range.tx_order_start + 1)
        .sum();
    let exp_txs = if let Some(origin_tx_order_end) = origin_tx_order_end {
        last_order - origin_tx_order_end
    } else {
        last_order
    };
    assert_eq!(
        exp_txs,
        total_txs,
        "Test case {}: total txs in new blocks mismatch, expected {}, got {}. last order: {}; origin tx order end: {:?}. block ranges: {:#?}",
        test_case,
        exp_txs,
        total_txs,
        last_order,
        origin_tx_order_end,
        block_ranges,
    );
}
