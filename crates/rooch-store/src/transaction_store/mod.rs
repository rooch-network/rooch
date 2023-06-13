// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use parking_lot::RwLock;
use rooch_types::transaction::{AbstractTransaction, TypedTransaction};
use rooch_types::H256;
use std::sync::Arc;

pub trait TransactionStore {
    fn add(&mut self, transaction: TypedTransaction);
    fn get_by_hash(&self, hash: H256) -> Option<TypedTransaction>;
    fn get_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction>;
}

pub struct TransactionDB {
    transaction_db: InMemoryStore,
}

impl TransactionDB {
    pub fn new_with_memory_store() -> Self {
        Self {
            transaction_db: InMemoryStore::default(),
        }
    }

    pub fn add(&mut self, transaction: TypedTransaction) {
        self.transaction_db.add(transaction);
    }

    pub fn get_by_hash(&self, hash: H256) -> Option<TypedTransaction> {
        self.transaction_db.get_by_hash(hash)
    }

    pub fn get_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction> {
        self.transaction_db.get_by_index(start, limit)
    }
}

#[derive(Default, Clone)]
pub struct InMemoryStore {
    inner: Arc<RwLock<Vec<TypedTransaction>>>,
}

impl TransactionStore for InMemoryStore {
    fn add(&mut self, transaction: TypedTransaction) {
        let mut inner = self.inner.write();
        inner.push(transaction);
    }

    fn get_by_hash(&self, hash: H256) -> Option<TypedTransaction> {
        let inner = self.inner.read();
        inner.iter().find(|tx| tx.tx_hash() == hash).cloned()
    }

    fn get_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction> {
        let inner = self.inner.read();
        let end = std::cmp::min((start + limit) as usize, inner.len());
        inner[start as usize..end].to_vec()
    }
}
