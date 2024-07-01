// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state::ObjectState;
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone)]
pub struct GenesisInfo {
    /// genesis package hash
    pub genesis_package_hash: H256,
    /// lastest state root hash
    pub root: ObjectState,
    /// genesis binary
    pub genesis_bin: Vec<u8>,
}

impl fmt::Debug for GenesisInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GenesisInfo {{ genesis_package_hash: {:?}, state_root_hash: {:?}, size: {} }}",
            self.genesis_package_hash,
            self.root.state_root(),
            self.root.size()
        )
    }
}

impl fmt::Display for GenesisInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GenesisInfo {{ genesis_package_hash: {:?}, state_root_hash: {:?}, size: {} }}",
            self.genesis_package_hash,
            self.root.state_root(),
            self.root.size()
        )
    }
}

impl GenesisInfo {
    pub fn new(genesis_package_hash: H256, root: ObjectState, genesis_bin: Vec<u8>) -> Self {
        GenesisInfo {
            genesis_package_hash,
            root,
            genesis_bin,
        }
    }
}
