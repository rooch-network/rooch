// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TX_ACCUMULATOR_NODE_PREFIX_NAME;
use accumulator::{AccumulatorNode, AccumulatorTreeStore};
use anyhow::Result;
use moveos_types::h256::H256;
use raw_store::{derive_store, CodecKVStore, StoreInstance};

derive_store!(
    TransactionAccumulatorStore,
    H256,
    AccumulatorNode,
    TX_ACCUMULATOR_NODE_PREFIX_NAME
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
