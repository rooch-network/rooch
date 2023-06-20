// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use parking_lot::RwLock;
use rooch_types::transaction::{
    AbstractTransaction, TransactionSequenceInfo, TransactionSequenceMapping, TypedTransaction,
};
use rooch_types::H256;
use std::collections::BTreeMap;
use std::sync::Arc;

pub trait TransactionStore {
    fn save_transaction(&mut self, transaction: TypedTransaction);
    fn get_tx_by_hash(&self, hash: H256) -> Option<TypedTransaction>;
    fn get_tx_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction>;

    fn save_tx_seq_info(&self, tx_seq_info: TransactionSequenceInfo);
    fn get_tx_seq_infos_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Vec<TransactionSequenceInfo>;
    fn save_tx_seq_info_mapping(&self, tx_order: u128, tx_hash: H256);
    fn get_tx_seq_mapping_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Vec<TransactionSequenceMapping>;
}

#[derive(Clone)]
pub struct TransactionDB {
    transaction_db: InMemoryStore,
}

impl TransactionDB {
    pub fn new_with_memory_store() -> Self {
        Self {
            transaction_db: InMemoryStore::default(),
        }
    }

    pub fn save_transaction(&mut self, transaction: TypedTransaction) {
        self.transaction_db.save_transaction(transaction);
    }

    pub fn get_tx_by_hash(&self, hash: H256) -> Option<TypedTransaction> {
        self.transaction_db.get_tx_by_hash(hash)
    }

    pub fn get_tx_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction> {
        self.transaction_db.get_tx_by_index(start, limit)
    }

    pub fn save_tx_seq_info(&self, tx_seq_info: TransactionSequenceInfo) {
        self.transaction_db.save_tx_seq_info(tx_seq_info);
    }

    pub fn get_tx_seq_infos_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Vec<TransactionSequenceInfo> {
        self.transaction_db
            .get_tx_seq_infos_by_tx_order(cursor, limit)
    }

    pub fn save_tx_seq_info_mapping(&self, tx_order: u128, tx_hash: H256) {
        self.transaction_db
            .save_tx_seq_info_mapping(tx_order, tx_hash)
    }

    pub fn get_tx_seq_mapping_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Vec<TransactionSequenceMapping> {
        self.transaction_db
            .get_tx_seq_mapping_by_tx_order(cursor, limit)
    }
}

#[derive(Default, Clone)]
pub struct InMemoryStore {
    inner_tx: Arc<RwLock<Vec<TypedTransaction>>>,
    inner_tx_seq_info: Arc<RwLock<BTreeMap<u128, TransactionSequenceInfo>>>,
    inner_tx_index: Arc<RwLock<BTreeMap<u128, H256>>>,
}

impl TransactionStore for InMemoryStore {
    fn save_transaction(&mut self, transaction: TypedTransaction) {
        let mut inner = self.inner_tx.write();
        inner.push(transaction);
    }

    fn get_tx_by_hash(&self, hash: H256) -> Option<TypedTransaction> {
        let inner = self.inner_tx.read();
        inner.iter().find(|tx| tx.tx_hash() == hash).cloned()
    }

    fn get_tx_by_index(&self, start: u64, limit: u64) -> Vec<TypedTransaction> {
        let inner = self.inner_tx.read();
        let end = std::cmp::min((start + limit) as usize, inner.len());
        inner[start as usize..end].to_vec()
    }

    fn save_tx_seq_info(&self, tx_seq_info: TransactionSequenceInfo) {
        let mut locked = self.inner_tx_seq_info.write();
        locked.insert(tx_seq_info.tx_order, tx_seq_info);
    }

    fn get_tx_seq_infos_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Vec<TransactionSequenceInfo> {
        let start = cursor.unwrap_or(0);
        let end = start + (limit as u128);
        let rw_locks = self.inner_tx_seq_info.read();
        let data = rw_locks
            .iter()
            .filter(|(tx_order, _)| {
                if Option::is_some(&cursor) {
                    **tx_order > start && **tx_order <= end
                } else {
                    **tx_order >= start && **tx_order < end
                }
            })
            .map(|(_, e)| e.clone())
            .collect::<Vec<_>>();
        data
    }

    fn save_tx_seq_info_mapping(&self, tx_order: u128, tx_hash: H256) {
        let mut locked = self.inner_tx_index.write();
        locked.insert(tx_order, tx_hash);
    }

    fn get_tx_seq_mapping_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Vec<TransactionSequenceMapping> {
        let start = cursor.unwrap_or(0);
        let end = start + (limit as u128);
        let rw_locks = self.inner_tx_index.read();
        let data = rw_locks
            .iter()
            .filter(|(tx_order, _)| {
                if Option::is_some(&cursor) {
                    **tx_order > start && **tx_order <= end
                } else {
                    **tx_order >= start && **tx_order < end
                }
            })
            .map(|(tx_order, tx_hash)| TransactionSequenceMapping::new(*tx_order, *tx_hash))
            .collect::<Vec<_>>();
        data
    }
}
