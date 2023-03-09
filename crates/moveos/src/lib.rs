// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use types::transaction::AbstractTransaction;

pub mod moveos;
pub mod types;
mod vm;

pub struct ValidatorResult {}

pub struct ExecutorResult {}

pub trait TransactionValidator {
    fn validate_transaction<T: AbstractTransaction>(&self, transaction: T) -> ValidatorResult;
}

pub trait TransactionExecutor {
    fn execute_transaction<T: AbstractTransaction>(&self, transaction: T) -> ExecutorResult;
}
