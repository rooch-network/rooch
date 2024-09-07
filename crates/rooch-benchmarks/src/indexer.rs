// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_types::indexer::state::IndexerObjectState;
use rooch_types::test_utils::{
    random_new_object_states_with_size_and_tx_order, random_update_object_states,
};

pub fn prepare_indexer_object_states_with_tx_order(
    nums: usize,
    tx_order: u64,
) -> Vec<IndexerObjectState> {
    random_new_object_states_with_size_and_tx_order(nums, tx_order)
}

pub fn gen_indexer_object_states_with_tx_order(
    nums: usize,
    tx_order: u64,
) -> Vec<IndexerObjectState> {
    let mut new_object_states = random_new_object_states_with_size_and_tx_order(nums, tx_order);
    let need_update_object_states = new_object_states
        .iter()
        .enumerate()
        .filter(|(idx, _state)| idx % 3 == 0)
        .map(|(_, state)| state.clone())
        .collect::<Vec<_>>();
    let mut update_object_states = random_update_object_states(need_update_object_states);
    new_object_states.append(&mut update_object_states);
    new_object_states
}
