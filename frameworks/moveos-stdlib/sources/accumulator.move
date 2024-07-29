// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::accumulator {

    #[data_struct]
    struct AccumulatorInfo has copy, drop, store{
        /// The tx accumulator root after the tx is append to the accumulator.
        tx_accumulator_root: vector<u8>,
        /// Frozen subtree roots of the accumulator.
        tx_accumulator_frozen_subtree_roots: vector<vector<u8>>,
        /// The total number of leaves in the accumulator.
        tx_accumulator_num_leaves: u64,
        /// The total number of nodes in the accumulator.
        tx_accumulator_num_nodes: u64,
    }
}