// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transaction {

    #[data_struct]
    struct TransactionSequenceInfo has copy, drop, store{
        /// The tx order
        tx_order: u64,
        /// The tx order signature, it is the signature of the sequencer to commit the tx order.
        tx_order_signature: vector<u8>,
        /// The tx accumulator root after the tx is append to the accumulator.
        tx_accumulator_root: vector<u8>,
        /// The timestamp of the sequencer when the tx is sequenced, in millisecond.
        tx_timestamp: u64,

        /// Frozen subtree roots of the accumulator.
        tx_accumulator_frozen_subtree_roots: vector<vector<u8>>,
        /// The total number of leaves in the accumulator.
        tx_accumulator_num_leaves: u64,
        /// The total number of nodes in the accumulator.
        tx_accumulator_num_nodes: u64,
    }

    public fun tx_order(self: &TransactionSequenceInfo): u64 {
        self.tx_order
    }

    public fun tx_order_signature(self: &TransactionSequenceInfo): vector<u8> {
        self.tx_order_signature
    }

    public fun tx_accumulator_root(self: &TransactionSequenceInfo): vector<u8> {
        self.tx_accumulator_root
    }

    public fun tx_timestamp(self: &TransactionSequenceInfo): u64 {
        self.tx_timestamp
    }
}