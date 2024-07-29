// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transaction {

    use std::option;
    use moveos_std::accumulator::AccumulatorInfo;

    #[deprecated]
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
    }

    #[data_struct]
    struct TransactionSequenceInfoV2 has copy, drop, store{
        /// The tx order
        tx_order: u64,
        /// The tx order signature, it is the signature of the sequencer to commit the tx order.
        tx_order_signature: vector<u8>,
        /// The tx accumulator root after the tx is append to the accumulator.
        tx_accumulator_root: vector<u8>,
        /// The timestamp of the sequencer when the tx is sequenced, in millisecond.
        tx_timestamp: u64,

        // /// Frozen subtree roots of the accumulator.
        // tx_accumulator_frozen_subtree_roots: vector<vector<u8>>,
        // /// The total number of leaves in the accumulator.
        // tx_accumulator_num_leaves: u64,
        // /// The total number of nodes in the accumulator.
        // tx_accumulator_num_nodes: u64,

        tx_accumulator_info: option::Option<AccumulatorInfo>,
    }

    #[deprecated]
    public fun tx_order(self: &TransactionSequenceInfo): u64 {
        self.tx_order
    }

    #[deprecated]
    public fun tx_order_signature(self: &TransactionSequenceInfo): vector<u8> {
        self.tx_order_signature
    }

    #[deprecated]
    public fun tx_accumulator_root(self: &TransactionSequenceInfo): vector<u8> {
        self.tx_accumulator_root
    }

    #[deprecated]
    public fun tx_timestamp(self: &TransactionSequenceInfo): u64 {
        self.tx_timestamp
    }

    public fun get_tx_order(self: &TransactionSequenceInfoV2): u64 {
        self.tx_order
    }

    public fun get_tx_order_signature(self: &TransactionSequenceInfoV2): vector<u8> {
        self.tx_order_signature
    }

    public fun get_tx_accumulator_root(self: &TransactionSequenceInfoV2): vector<u8> {
        self.tx_accumulator_root
    }

    public fun get_tx_timestamp(self: &TransactionSequenceInfoV2): u64 {
        self.tx_timestamp
    }
}