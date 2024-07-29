// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::h256::{ACCUMULATOR_PLACEHOLDER_HASH, H256};
use crate::state::{MoveState, MoveStructState, MoveStructType};
use bcs_ext::Sample;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl MoveStructType for AccumulatorInfo {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("accumulator");
    const STRUCT_NAME: &'static IdentStr = ident_str!("AccumulatorInfo");
}

impl MoveStructState for AccumulatorInfo {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            // move_core_types::value::MoveTypeLayout::Vector(Box::new(
            //     move_core_types::value::MoveTypeLayout::Vector(Box::new(
            //         move_core_types::value::MoveTypeLayout::U8,
            //     )),
            // )),
            Vec::<Vec<u8>>::type_layout(),
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::U64,
        ])
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

impl fmt::Display for AccumulatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccumulatorInfo[accumulator_root: {:?}, frozen_subtree_roots: {:?}, num_leaves: {}, num_nodes: {}]",
            self.accumulator_root,
            self.frozen_subtree_roots,
            self.num_leaves,
            self.num_nodes
        )
    }
}
