// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;
use moveos_types::transaction::TransactionExecutionInfo;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

use raw_store::derive_store;
use raw_store::ValueCodec;
use crate::TRANSACTION_PREFIX_NAME;

derive_store!(TransactionDBStore, H256, TransactionExecutionInfo, TRANSACTION_PREFIX_NAME);

impl ValueCodec for TransactionExecutionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

pub trait TransactionStore {
    fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) -> Result<()> ;
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

impl TransactionStore for TransactionDBStore {
    fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) -> Result<()> {
        // let mut locked = self.inner_tx_exec_info.write();
        // locked.insert(tx_exec_info.tx_hash, tx_exec_info);

        self.put(txn_info.id(), txn_info)
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


    // fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>> {
    //     self.get(txn_hash)
    // }
    //
    // fn save_transaction(&self, txn_info: Transaction) -> Result<()> {
    //     self.put(txn_info.id(), txn_info)
    // }
    //
    // fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<()> {
    //     let batch =
    //         CodecWriteBatch::new_puts(txn_vec.into_iter().map(|txn| (txn.id(), txn)).collect());
    //     self.write_batch(batch)
    // }
    //
    // fn get_transactions(&self, txn_hash_vec: Vec<HashValue>) -> Result<Vec<Option<Transaction>>> {
    //     self.multiple_get(txn_hash_vec)
    // }
}
