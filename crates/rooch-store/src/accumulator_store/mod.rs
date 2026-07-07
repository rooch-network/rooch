// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME;
use accumulator::{AccumulatorNode, AccumulatorTreeStore};
use anyhow::Result;
use moveos_types::h256::H256;
use raw_store::{derive_store, CodecKVStore, StoreInstance};

derive_store!(
    TransactionAccumulatorStore,
    H256,
    AccumulatorNode,
    TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME
);

#[derive(Clone)]
pub struct AccumulatorStore<S>
where
    S: CodecKVStore<H256, AccumulatorNode>,
{
    store: S,
}

impl AccumulatorStore<TransactionAccumulatorStore> {
    pub fn new_transaction_accumulator_store(
        instance: StoreInstance,
    ) -> AccumulatorStore<TransactionAccumulatorStore> {
        Self {
            store: TransactionAccumulatorStore::new(instance),
        }
    }

    pub fn iter_leaves(&self, max_leaf_count: u64) -> Result<Vec<(u64, H256)>> {
        let mut iter = self.store.iter()?;
        iter.seek_to_first();

        let mut leaves = Vec::new();
        for item in iter {
            let (_hash, node) = item?;
            match node {
                AccumulatorNode::Empty => {}
                AccumulatorNode::Internal(_) => {}
                AccumulatorNode::Leaf(leaf) => {
                    let index = leaf
                        .index()
                        .to_leaf_index()
                        .ok_or_else(|| anyhow::anyhow!("accumulator leaf has non-leaf index"))?;
                    if index < max_leaf_count {
                        leaves.push((index, leaf.value()));
                    }
                }
            }
        }
        leaves.sort_unstable_by_key(|(index, _)| *index);
        Ok(leaves)
    }
}

impl<S> AccumulatorTreeStore for AccumulatorStore<S>
where
    S: CodecKVStore<H256, AccumulatorNode>,
{
    fn get_node(&self, hash: H256) -> Result<Option<AccumulatorNode>> {
        self.store.kv_get(hash)
    }

    fn multiple_get(&self, keys: Vec<H256>) -> Result<Vec<Option<AccumulatorNode>>> {
        self.store.multiple_get(keys)
    }

    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.store.kv_put(node.hash(), node)
    }

    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<()> {
        self.store
            .put_all(nodes.into_iter().map(|node| (node.hash(), node)).collect())
    }

    fn delete_nodes(&self, node_hash_vec: Vec<H256>) -> Result<()> {
        self.store.delete_all(node_hash_vec)
    }
}
