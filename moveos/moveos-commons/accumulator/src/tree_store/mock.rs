// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{AccumulatorNode, AccumulatorTreeStore};
use anyhow::{bail, Result};
use moveos_types::h256::H256;
use parking_lot::Mutex;
use std::collections::HashMap;

pub struct MockAccumulatorStore {
    node_store: Mutex<HashMap<H256, AccumulatorNode>>,
}

impl MockAccumulatorStore {
    pub fn new() -> MockAccumulatorStore {
        MockAccumulatorStore {
            node_store: Mutex::new(HashMap::new()),
        }
    }

    pub fn copy_from(&self) -> Self {
        Self {
            node_store: Mutex::new(self.node_store.lock().clone()),
        }
    }
}

impl Default for MockAccumulatorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AccumulatorTreeStore for MockAccumulatorStore {
    fn get_node(&self, hash: H256) -> Result<Option<AccumulatorNode>> {
        let map = self.node_store.lock();
        match map.get(&hash) {
            Some(node) => Ok(Some(node.clone())),
            None => bail!("get node is null: {}", hash),
        }
    }

    fn multiple_get(&self, _hash_vec: Vec<H256>) -> Result<Vec<Option<AccumulatorNode>>> {
        unimplemented!()
    }

    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.node_store.lock().insert(node.hash(), node);
        Ok(())
    }

    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<()> {
        let mut store = self.node_store.lock();
        for node in nodes {
            store.insert(node.hash(), node);
        }
        Ok(())
    }

    fn delete_nodes(&self, node_hash_vec: Vec<H256>) -> Result<()> {
        for hash in node_hash_vec {
            self.node_store.lock().remove(&hash);
        }
        Ok(())
    }
}
