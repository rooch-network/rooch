// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;
use moveos_types::transaction::TransactionExecutionInfo;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

pub trait TransactionStore {
    fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo);
    fn get_tx_exec_info(&self, tx_hash: H256) -> Option<TransactionExecutionInfo>;
    fn multi_get_tx_exec_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Vec<Option<TransactionExecutionInfo>>;
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

    pub fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) {
        self.transaction_db.save_tx_exec_info(tx_exec_info);
    }

    pub fn get_tx_exec_info(&self, tx_hash: H256) -> Option<TransactionExecutionInfo> {
        self.transaction_db.get_tx_exec_info(tx_hash)
    }

    pub fn multi_get_tx_exec_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Vec<Option<TransactionExecutionInfo>> {
        self.transaction_db.multi_get_tx_exec_infos(tx_hashes)
    }
}

#[derive(Default, Clone)]
pub struct InMemoryStore {
    inner_tx_exec_info: Arc<RwLock<BTreeMap<H256, TransactionExecutionInfo>>>,
}

impl TransactionStore for InMemoryStore {
    fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) {
        let mut locked = self.inner_tx_exec_info.write();
        locked.insert(tx_exec_info.tx_hash, tx_exec_info);
    }

    fn get_tx_exec_info(&self, tx_hash: H256) -> Option<TransactionExecutionInfo> {
        let rw_locks = self.inner_tx_exec_info.read();
        let data = rw_locks.get(&tx_hash);
        data.cloned()
    }

    fn multi_get_tx_exec_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Vec<Option<TransactionExecutionInfo>> {
        let rw_locks = self.inner_tx_exec_info.read();

        let data = tx_hashes
            .iter()
            .map(|tx_hash| rw_locks.get(tx_hash).cloned())
            .collect::<Vec<_>>();
        data
    }
}
