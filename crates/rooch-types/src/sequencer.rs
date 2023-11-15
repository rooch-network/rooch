// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct SequencerOrder {
    pub last_order: u64,
}

impl fmt::Display for SequencerOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SequencerOrder {{ last_order: {} }}", self.last_order)
    }
}

impl SequencerOrder {
    pub fn new(last_order: u64) -> Self {
        SequencerOrder { last_order }
    }
}
