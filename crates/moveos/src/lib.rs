// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use types::transaction::AbstractTransaction;

pub mod moveos;
#[cfg(test)]
mod tests;
pub mod types;
mod vm;

/// Define the Rooch address with `0x1`
const MOS_ADDRESS: &str = "0x1";

pub const fn mos_address() -> &'static str {
    MOS_ADDRESS
}

pub struct ValidatorResult {}

pub struct ExecutorResult {}

pub trait TransactionValidator {
    fn validate_transaction<T: AbstractTransaction>(&self, transaction: T) -> ValidatorResult;
}

pub trait TransactionExecutor {
    fn execute_transaction<T: AbstractTransaction>(&self, transaction: T) -> ExecutorResult;
}
