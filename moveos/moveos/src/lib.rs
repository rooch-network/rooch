// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod moveos;
#[cfg(test)]
mod tests;
pub mod vm;

pub struct ValidatorResult {}

pub struct ExecutorResult {}

pub trait TransactionValidator {
    fn validate_transaction<T>(&self, transaction: T) -> ValidatorResult;
}

pub trait TransactionExecutor {
    fn execute_transaction<T>(&self, transaction: T) -> ExecutorResult;
}
