// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::RoochStore;
use accumulator::node_index::NodeIndex;
use accumulator::{Accumulator, AccumulatorNode, AccumulatorTreeStore, MerkleAccumulator};
use moveos_types::h256::H256;

#[tokio::test]
async fn test_accumulator_store() {
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

#[tokio::test]
async fn accumulator_pop_unsaved() {
    let (rooch_store, _) = RoochStore::mock_rooch_store().unwrap();

    let tx_accumulator =
        MerkleAccumulator::new_empty(rooch_store.get_transaction_accumulator_store());
    let leaves = vec![H256::random(), H256::random(), H256::random()];
    let _root = tx_accumulator.append(&leaves).unwrap();
    let accumulator_info = tx_accumulator.get_info();

    let num_leaves = accumulator_info.num_leaves;
    let tx_accumulator_unsaved = MerkleAccumulator::new_with_info(
        accumulator_info.clone(),
        rooch_store.get_transaction_accumulator_store(),
    );
    for i in 0..num_leaves - 1 {
        let leaf = tx_accumulator_unsaved.get_leaf(i);
        assert!(leaf.is_err());
    }
    // last leaf should be in frozen_subtree_roots, so it should be found.
    assert_eq!(
        leaves[num_leaves as usize - 1],
        tx_accumulator_unsaved
            .get_leaf(num_leaves - 1)
            .unwrap()
            .unwrap()
    );

    let unsaved_nodes = tx_accumulator.pop_unsaved_nodes();
    rooch_store
        .get_transaction_accumulator_store()
        .save_nodes(unsaved_nodes.unwrap())
        .unwrap();
    let tx_accumulator_saved = MerkleAccumulator::new_with_info(
        accumulator_info,
        rooch_store.get_transaction_accumulator_store(),
    );
    for i in 0..num_leaves {
        let leaf = tx_accumulator_saved.get_leaf(i);
        assert!(leaf.is_ok());
        assert_eq!(leaves[i as usize], leaf.unwrap().unwrap());
    }
}
