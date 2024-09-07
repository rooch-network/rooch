// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone)]
pub struct GenesisInfo {
    /// genesis package hash
    pub genesis_package_hash: H256,
    /// genesis binary
    pub genesis_bin: Vec<u8>,
}

impl fmt::Debug for GenesisInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GenesisInfo {{ genesis_package_hash: {:?}}}",
            self.genesis_package_hash,
        )
    }
}

impl fmt::Display for GenesisInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GenesisInfo {{ genesis_package_hash: {:?}}}",
            self.genesis_package_hash,
        )
    }
}

impl GenesisInfo {
    pub fn new(genesis_package_hash: H256, genesis_bin: Vec<u8>) -> Self {
        GenesisInfo {
            genesis_package_hash,
            genesis_bin,
        }
    }
}
