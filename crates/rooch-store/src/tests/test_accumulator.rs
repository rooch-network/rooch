// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::RoochStore;
use accumulator::node_index::NodeIndex;
use accumulator::{AccumulatorNode, AccumulatorTreeStore};
use moveos_types::h256::H256;

#[test]
fn test_accumulator_store() {
    let (rooch_store, _) = RoochStore::mock_rooch_store().unwrap();

    let acc_node = AccumulatorNode::new_leaf(NodeIndex::from_inorder_index(1), H256::random());
    let node_hash = acc_node.hash();
    rooch_store
        .transaction_accumulator_store
        .save_node(acc_node.clone())
        .unwrap();
    let acc_node2 = rooch_store
        .transaction_accumulator_store
        .get_node(node_hash)
        .unwrap()
        .unwrap();
    assert_eq!(acc_node, acc_node2);
}
