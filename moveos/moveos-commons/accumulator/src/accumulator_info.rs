// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::Sample;
use moveos_types::h256::{ACCUMULATOR_PLACEHOLDER_HASH, H256};
use serde::{Deserialize, Serialize};

/// `AccumulatorInfo` is the object we store in the storage. It consists of the
/// info that we can create MerkleAccumulator.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccumulatorInfo {
    /// Accumulator root hash
    pub accumulator_root: H256,
    /// Frozen subtree roots of this accumulator.
    pub frozen_subtree_roots: Vec<H256>,
    /// The total number of leaves in this accumulator.
    pub num_leaves: u64,
    /// The total number of nodes in this accumulator.
    pub num_nodes: u64,
}

impl AccumulatorInfo {
    pub fn new(
        accumulator_root: H256,
        frozen_subtree_roots: Vec<H256>,
        num_leaves: u64,
        num_nodes: u64,
    ) -> Self {
        AccumulatorInfo {
            accumulator_root,
            frozen_subtree_roots,
            num_leaves,
            num_nodes,
        }
    }

    pub fn get_accumulator_root(&self) -> &H256 {
        &self.accumulator_root
    }

    pub fn get_frozen_subtree_roots(&self) -> &Vec<H256> {
        &self.frozen_subtree_roots
    }

    pub fn get_num_leaves(&self) -> u64 {
        self.num_leaves
    }

    pub fn get_num_nodes(&self) -> u64 {
        self.num_nodes
    }
}

impl Default for AccumulatorInfo {
    fn default() -> Self {
        AccumulatorInfo {
            accumulator_root: *ACCUMULATOR_PLACEHOLDER_HASH,
            frozen_subtree_roots: Vec::new(),
            num_leaves: 0,
            num_nodes: 0,
        }
    }
}

impl Sample for AccumulatorInfo {
    fn sample() -> Self {
        Self::default()
    }
}
