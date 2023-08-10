// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Ok, Result};
use moveos_common::accumulator::Accumulator;
use raw_store::CodecKVStore;

use crate::ACCUMULATOR_PREFIX_NAME;
use raw_store::derive_store;

const ACCUMULATOR_KEY: &str = "accumulator";

derive_store!(
    AccumulatorDBStore,
    String,
    Accumulator,
    ACCUMULATOR_PREFIX_NAME
);

pub trait AccumulatorStore {
    fn save_accumulator(&self, accumulator: Accumulator) -> Result<()>;
    fn get_accumulator(&self) -> Result<Option<Accumulator>>;
}

impl AccumulatorStore for AccumulatorDBStore {
    fn save_accumulator(&self, accumulator: Accumulator) -> Result<()> {
        self.kv_put(ACCUMULATOR_KEY.to_string(), accumulator)?;
        Ok(())
    }

    fn get_accumulator(&self) -> Result<Option<Accumulator>> {
        self.kv_get(ACCUMULATOR_KEY.to_string())
    }
}
