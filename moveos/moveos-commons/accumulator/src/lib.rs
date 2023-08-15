// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator_info::AccumulatorInfo;
use crate::node_index::NodeIndex;
use crate::proof::AccumulatorProof;
use crate::tree::AccumulatorTree;
use anyhow::{format_err, Result};
use moveos_types::h256::H256;
pub use node::AccumulatorNode;
use parking_lot::Mutex;
#[cfg(test)]
use std::collections::HashMap;
use std::sync::Arc;
pub use tree_store::AccumulatorTreeStore;

pub mod accumulator_info;
pub mod inmemory;
pub mod node;
pub mod node_index;
pub mod proof;
#[cfg(test)]
mod tests;
mod tree;
pub mod tree_store;

pub type LeafCount = u64;
pub type NodeCount = u64;

pub const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
pub const MAX_ACCUMULATOR_LEAVES: LeafCount = 1 << MAX_ACCUMULATOR_PROOF_DEPTH;
pub const MAX_CACHE_SIZE: usize = 65535;

/// accumulator method define
pub trait Accumulator {
    /// Append leaves and return new root
    fn append(&self, leaves: &[H256]) -> Result<H256>;
    /// Get leaf node by index.
    fn get_leaf(&self, leaf_index: u64) -> Result<Option<H256>>;
    /// Batch get leaves by index.
    fn get_leaves(&self, start_index: u64, reverse: bool, max_size: u64) -> Result<Vec<H256>>;
    /// Get node by position.
    fn get_node_by_position(&self, position: u64) -> Result<Option<H256>>;
    /// Get proof by leaf index.
    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>>;
    /// Flush node to storage.
    fn flush(&self) -> Result<()>;
    /// Get current accumulator tree root hash.
    fn root_hash(&self) -> H256;
    /// Get current accumulator tree number of leaves.
    fn num_leaves(&self) -> u64;
    /// Get current accumulator tree number of nodes.
    fn num_nodes(&self) -> u64;
    /// Get frozen subtree roots.
    fn get_frozen_subtree_roots(&self) -> Vec<H256>;
    /// Get accumulator info
    fn get_info(&self) -> AccumulatorInfo;
}

/// MerkleAccumulator is a accumulator algorithm implement and it is stateless.
pub struct MerkleAccumulator {
    tree: Mutex<AccumulatorTree>,
}

impl MerkleAccumulator {
    pub fn new(
        root_hash: H256,
        frozen_subtree_roots: Vec<H256>,
        num_leaves: LeafCount,
        num_notes: NodeCount,
        node_store: Arc<dyn AccumulatorTreeStore>,
    ) -> Self {
        Self {
            tree: Mutex::new(AccumulatorTree::new(
                frozen_subtree_roots,
                num_leaves,
                num_notes,
                root_hash,
                node_store,
            )),
        }
    }

    pub fn new_with_info(
        acc_info: AccumulatorInfo,
        node_store: Arc<dyn AccumulatorTreeStore>,
    ) -> Self {
        Self {
            tree: Mutex::new(AccumulatorTree::new(
                acc_info.frozen_subtree_roots,
                acc_info.num_leaves,
                acc_info.num_nodes,
                acc_info.accumulator_root,
                node_store,
            )),
        }
    }

    pub fn new_empty(node_store: Arc<dyn AccumulatorTreeStore>) -> Self {
        Self {
            tree: Mutex::new(AccumulatorTree::new_empty(node_store)),
        }
    }

    /// Fork a new accumulator base on current accumulator
    pub fn fork(&self, info: Option<AccumulatorInfo>) -> MerkleAccumulator {
        Self::new_with_info(
            info.unwrap_or_else(|| self.get_info()),
            self.tree.lock().store.clone(),
        )
    }

    #[cfg(test)]
    pub fn get_index_frozen_subtrees(&self) -> HashMap<NodeIndex, H256> {
        self.tree.lock().get_index_frozen_subtrees()
    }
}

impl Accumulator for MerkleAccumulator {
    fn append(&self, new_leaves: &[H256]) -> Result<H256> {
        let mut tree_guard = self.tree.lock();
        let root_hash = tree_guard.append(new_leaves)?;
        Ok(root_hash)
    }

    fn get_leaf(&self, leaf_index: u64) -> Result<Option<H256>> {
        self.tree
            .lock()
            .get_node_hash(NodeIndex::from_leaf_index(leaf_index))
    }

    fn get_leaves(&self, start_index: u64, reverse: bool, max_size: u64) -> Result<Vec<H256>> {
        let mut tree = self.tree.lock();
        if reverse {
            let mut end = start_index.saturating_add(1);
            if end > tree.num_leaves {
                end = tree.num_leaves;
            }
            let begin = end.saturating_sub(max_size);
            (begin..end)
                .rev()
                .map(|idx| {
                    tree.get_node_hash(NodeIndex::from_leaf_index(idx))?
                        .ok_or_else(|| {
                            format_err!(
                                "Can not find accumulator leaf by index: {}, num_leaves:{}",
                                idx,
                                tree.num_leaves
                            )
                        })
                })
                .collect()
        } else {
            let mut end = start_index.saturating_add(max_size);
            if end > tree.num_leaves {
                end = tree.num_leaves;
            }
            (start_index..end)
                .map(|idx| {
                    tree.get_node_hash(NodeIndex::from_leaf_index(idx))?
                        .ok_or_else(|| {
                            format_err!(
                                "Can not find accumulator leaf by index: {}, num_leaves: {}",
                                idx,
                                tree.num_leaves
                            )
                        })
                })
                .collect()
        }
    }

    fn get_node_by_position(&self, position: u64) -> Result<Option<H256>> {
        self.tree
            .lock()
            .get_node_hash(NodeIndex::from_inorder_index(position))
    }

    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>> {
        let mut tree_guard = self.tree.lock();
        if leaf_index > tree_guard.num_leaves {
            return Ok(None);
        }

        let siblings = tree_guard.get_siblings(leaf_index, |_p| true)?;
        Ok(Some(AccumulatorProof::new(siblings)))
    }

    fn flush(&self) -> Result<()> {
        self.tree.lock().flush()
    }

    fn root_hash(&self) -> H256 {
        self.tree.lock().root_hash
    }
    fn num_leaves(&self) -> u64 {
        self.tree.lock().num_leaves
    }
    fn num_nodes(&self) -> u64 {
        self.tree.lock().num_nodes
    }

    fn get_frozen_subtree_roots(&self) -> Vec<H256> {
        self.tree.lock().get_frozen_subtree_roots()
    }

    fn get_info(&self) -> AccumulatorInfo {
        AccumulatorInfo::new(
            self.root_hash(),
            self.get_frozen_subtree_roots(),
            self.num_leaves(),
            self.num_nodes(),
        )
    }
}
