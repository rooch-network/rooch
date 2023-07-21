// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_types::h256::H256;
use moveos_types::transaction::TransactionExecutionInfo;
use raw_store::CodecKVStore;

use crate::TRANSACTION_PREFIX_NAME;
use raw_store::derive_store;

derive_store!(
    TransactionDBStore,
    H256,
    TransactionExecutionInfo,
    TRANSACTION_PREFIX_NAME
);

pub trait TransactionStore {
    fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) -> Result<()>;
    fn get_tx_exec_info(&self, tx_hash: H256) -> Result<Option<TransactionExecutionInfo>>;
    fn multi_get_tx_exec_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>>;
}

// pub struct TransactionDB {
//     transaction_db: InMemoryStore,
// }
//
// impl TransactionDB {
//     pub fn new_with_memory_store() -> Self {
//         Self {
//             transaction_db: InMemoryStore::default(),
//         }
//     }
//
//     pub fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) {
//         self.transaction_db.save_tx_exec_info(tx_exec_info);
//     }
//
//     pub fn get_tx_exec_info(&self, tx_hash: H256) -> Option<TransactionExecutionInfo> {
//         self.transaction_db.get_tx_exec_info(tx_hash)
//     }
//
//     pub fn multi_get_tx_exec_infos(
//         &self,
//         tx_hashes: Vec<H256>,
//     ) -> Vec<Option<TransactionExecutionInfo>> {
//         self.transaction_db.multi_get_tx_exec_infos(tx_hashes)
//     }
// }
//
// #[derive(Default, Clone)]
// pub struct InMemoryStore {
//     inner_tx_exec_info: Arc<RwLock<BTreeMap<H256, TransactionExecutionInfo>>>,
// }

impl TransactionStore for TransactionDBStore {
    fn save_tx_exec_info(&self, tx_exec_info: TransactionExecutionInfo) -> Result<()> {
        // let mut locked = self.inner_tx_exec_info.write();
        // locked.insert(tx_exec_info.tx_hash, tx_exec_info);
        self.kv_put(tx_exec_info.id(), tx_exec_info)
    }

    fn get_tx_exec_info(&self, tx_hash: H256) -> Result<Option<TransactionExecutionInfo>> {
        // let rw_locks = self.inner_tx_exec_info.read();
        // let data = rw_locks.get(&tx_hash);
        // data.cloned()
        self.kv_get(tx_hash)
    }

    fn multi_get_tx_exec_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        // let rw_locks = self.inner_tx_exec_info.read();
        //
        // let data = tx_hashes
        //     .iter()
        //     .map(|tx_hash| rw_locks.get(tx_hash).cloned())
        //     .collect::<Vec<_>>();
        // data
        self.multiple_get(tx_hashes)
    }
}
