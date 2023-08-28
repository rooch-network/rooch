// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct GenesisInfo {
    /// genesis package hash
    pub genesis_package_hash: H256,
    /// lastest state root hash
    pub state_root_hash: H256,
}

impl fmt::Display for GenesisInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GenesisInfo {{ genesis_package_hash: {}, state_root_hash: {} }}",
            self.genesis_package_hash, self.state_root_hash
        )
    }
}

impl GenesisInfo {

    pub fn new(genesis_package_hash: H256, state_root_hash: H256) -> Self {
        GenesisInfo {
            genesis_package_hash,
            state_root_hash,
        }
    }
}
