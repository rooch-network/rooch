// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use accumulator::accumulator_info::AccumulatorInfo;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone)]
pub struct SequencerInfo {
    pub last_order: u64,
    pub last_accumulator_info: AccumulatorInfo,
}

impl fmt::Display for SequencerInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SequencerInfo {{ last_order: {}, last_accumulator_info: {:?} }}",
            self.last_order, self.last_accumulator_info
        )
    }
}

impl SequencerInfo {
    pub fn new(last_order: u64, last_accumulator_info: AccumulatorInfo) -> Self {
        SequencerInfo {
            last_order,
            last_accumulator_info,
        }
    }
}
