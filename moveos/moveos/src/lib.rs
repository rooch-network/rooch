// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{ident_str, identifier::IdentStr};
use moveos_types::transaction::AbstractTransaction;

pub mod moveos;
#[cfg(test)]
mod tests;
pub mod vm;

pub const INIT_FN_NAME: &IdentStr = ident_str!("init");

pub struct ValidatorResult {}

pub struct ExecutorResult {}

pub trait TransactionValidator {
    fn validate_transaction<T: AbstractTransaction>(&self, transaction: T) -> ValidatorResult;
}

pub trait TransactionExecutor {
    fn execute_transaction<T: AbstractTransaction>(&self, transaction: T) -> ExecutorResult;
}
