// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use wasmer::RuntimeError;

#[derive(Debug)]
pub struct GasMeter {
    gas_limit: u64,
    gas_used: u64,
}

impl GasMeter {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            gas_used: 0,
        }
    }

    pub fn reset(&mut self) {
        self.gas_used = 0;
    }

    pub fn charge(&mut self, amount: u64) -> Result<(), RuntimeError> {
        if self.gas_used + amount > self.gas_limit {
            Err(RuntimeError::new("GAS limit exceeded"))
        } else {
            self.gas_used += amount;
            Ok(())
        }
    }

    pub fn used(&mut self) -> u64 {
        self.gas_used
    }
}
