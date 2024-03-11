// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

use crate::moveos_std::object::{ObjectEntity, RootObjectEntity};

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct StartupInfo {
    /// Global state root hash
    pub state_root: H256,
    /// Global state tree size
    pub size: u64,
}

impl fmt::Display for StartupInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StartupInfo {{")?;
        write!(f, "state_root_hash: {:?},", self.state_root)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl StartupInfo {
    pub fn new(state_root: H256, size: u64) -> Self {
        Self { state_root, size }
    }

    pub fn update_state_root(&mut self, new_state_root_hash: H256, size: u64) {
        self.state_root = new_state_root_hash;
        self.size = size;
    }

    pub fn get_state_root(&self) -> &H256 {
        &self.state_root
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn into_root_object(self) -> RootObjectEntity {
        ObjectEntity::root_object(self.state_root, self.size)
    }
}
