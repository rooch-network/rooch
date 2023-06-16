// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::transaction_store::TransactionDB;

pub mod transaction_store;

#[derive(Clone)]
pub struct RoochDB {
    pub transaction_store: TransactionDB,
}

impl RoochDB {
    pub fn new_with_memory_store() -> Self {
        Self {
            transaction_store: TransactionDB::new_with_memory_store(),
        }
    }

    pub fn get_transaction_store(&self) -> &TransactionDB {
        &self.transaction_store
    }
}
