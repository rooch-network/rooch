// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use statedb::StateDB;
use crate::{TransactionValidator, types::transaction::AbstractTransaction, TransactionExecutor};

pub struct MoveOS{

}

impl MoveOS {
    
    pub fn new(_db: StateDB) -> Self{
        todo!()
    }
}

impl TransactionValidator for MoveOS{
    fn validate_transaction<T:AbstractTransaction>(
        &self,
        _transaction: T,
    ) -> crate::ValidatorResult {
        todo!()
    }
}

impl TransactionExecutor for MoveOS{
    fn execute_transaction<T:AbstractTransaction>(
        &self,
        _transaction: T,
    ) -> crate::ExecutorResult {
        todo!()
    }
}