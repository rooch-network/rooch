// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node_index::{NodeIndex, G_NODE_ERROR_INDEX};
use anyhow::Result;
use moveos_types::h256::{sha3_256_of, ACCUMULATOR_PLACEHOLDER_HASH, H256};
use serde::{Deserialize, Serialize};

//TODO move to a more suitable crate.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccumulatorStoreType {
    Transaction,
    Block,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccumulatorNode {
    Internal(InternalNode),
    Leaf(LeafNode),
    Empty,
}

impl AccumulatorNode {
    pub fn new_internal(index: NodeIndex, left: H256, right: H256) -> Self {
        AccumulatorNode::Internal(InternalNode::new(index, left, right))
    }

    pub fn new_leaf(index: NodeIndex, value: H256) -> Self {
        AccumulatorNode::Leaf(LeafNode::new(index, value))
    }

    pub fn hash(&self) -> H256 {
        match self {
            AccumulatorNode::Internal(internal) => internal.hash(),
            AccumulatorNode::Leaf(leaf) => leaf.value(),
            AccumulatorNode::Empty => *ACCUMULATOR_PLACEHOLDER_HASH,
        }
    }

    pub fn index(&self) -> NodeIndex {
        match self {
            AccumulatorNode::Internal(internal) => internal.index(),
            AccumulatorNode::Leaf(leaf) => leaf.index(),
            AccumulatorNode::Empty => {
                // bail!("error for get index");
                *G_NODE_ERROR_INDEX
            }
        }
    }

    pub fn frozen(&mut self) -> Result<()> {
        let _node = match self {
            AccumulatorNode::Internal(internal) => internal.set_frozen(),
            _ => Ok(()),
        };
        Ok(())
    }

    pub fn is_frozen(&self) -> bool {
        match self {
            AccumulatorNode::Internal(internal) => internal.is_frozen,
            AccumulatorNode::Leaf(_) => true,
            AccumulatorNode::Empty => false,
        }
    }
}

/// An internal node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InternalNode {
    index: NodeIndex,
    left: H256,
    right: H256,
    is_frozen: bool,
}

impl InternalNode {
    pub fn new(index: NodeIndex, left: H256, right: H256) -> Self {
        InternalNode {
            index,
            left,
            right,
            is_frozen: right != *ACCUMULATOR_PLACEHOLDER_HASH,
        }
    }

    pub fn hash(&self) -> H256 {
        let mut bytes = self.left.0.to_vec();
        bytes.extend(self.right.0.to_vec());
        sha3_256_of(bytes.as_slice())
    }

    pub fn index(&self) -> NodeIndex {
        self.index
    }
    pub fn left(&self) -> H256 {
        self.left
    }
    pub fn right(&self) -> H256 {
        self.right
    }

    pub fn set_frozen(&mut self) -> Result<()> {
        self.is_frozen = true;
        Ok(())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct LeafNode {
    index: NodeIndex,
    hash: H256,
}

impl LeafNode {
    pub fn new(index: NodeIndex, hash: H256) -> Self {
        LeafNode { index, hash }
    }

    pub fn value(&self) -> H256 {
        self.hash
    }

    pub fn index(&self) -> NodeIndex {
        self.index
    }
}
