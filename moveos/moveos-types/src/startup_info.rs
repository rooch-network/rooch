// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct StartupInfo {
    /// lastest state root hash
    pub state_root_hash: H256,
}

impl fmt::Display for StartupInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StartupInfo {{")?;
        write!(f, "state_root_hash: {:?},", self.state_root_hash)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl StartupInfo {
    pub fn new(state_root_hash: H256) -> Self {
        Self { state_root_hash }
    }

    pub fn update_state_root_hash(&mut self, new_state_root_hash: H256) {
        self.state_root_hash = new_state_root_hash;
    }

    pub fn get_state_root_hash(&self) -> &H256 {
        &self.state_root_hash
    }
}
