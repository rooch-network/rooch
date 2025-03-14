// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{TRANSACTION_COLUMN_FAMILY_NAME, TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME};
use anyhow::Result;
use moveos_common::utils::to_bytes;
use moveos_types::h256::H256;
use raw_store::rocks::batch::WriteBatch;
use raw_store::traits::DBStore;
use raw_store::CodecKVStore;
use raw_store::{derive_store, StoreInstance};
use rooch_types::transaction::LedgerTransaction;

derive_store!(
    LedgerTransactionStore,
    H256,
    LedgerTransaction,
    TRANSACTION_COLUMN_FAMILY_NAME
);

derive_store!(
    TxSequenceInfoMappingStore,
    u64,
    H256,
    TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME
);

pub trait TransactionStore {
    fn remove_transaction(&self, tx_hash: H256, tx_order: u64) -> Result<()>;
    fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>>;
    fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<LedgerTransaction>>>;

    fn get_tx_hashes(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>>;

    fn get_tx_hashes_by_order(&self, cursor: Option<u64>, limit: u64) -> Result<Vec<Option<H256>>> {
        let start = cursor.unwrap_or(0);
        let end = start + limit;

        // Since tx order is strictly incremental, traversing the SMT Tree can be optimized into a multi get query to improve query performance.
        let tx_orders: Vec<_> = if cursor.is_some() {
            ((start + 1)..=end).collect()
        } else {
            (start..end).collect()
        };
        self.get_tx_hashes(tx_orders)
    }
}

#[derive(Clone)]
pub struct TransactionDBStore {
    tx_store: LedgerTransactionStore,
    tx_sequence_info_mapping_store: TxSequenceInfoMappingStore,
}

impl TransactionDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        TransactionDBStore {
            tx_store: LedgerTransactionStore::new(instance.clone()),
            tx_sequence_info_mapping_store: TxSequenceInfoMappingStore::new(instance.clone()),
        }
    }

    pub fn remove_transaction(&self, tx_hash: H256, tx_order: u64) -> Result<()> {
        let inner_store = self.tx_store.store.store();

        let mut write_batch = WriteBatch::new();
        write_batch.delete(to_bytes(&tx_hash).unwrap())?;
        write_batch.delete(to_bytes(&tx_order).unwrap())?;

        inner_store.write_batch_across_cfs(
            vec![
                TRANSACTION_COLUMN_FAMILY_NAME,
                TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
            ],
            write_batch,
            true,
        )?;
        Ok(())
    }

    pub fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>> {
        self.tx_store.kv_get(hash)
    }

    pub fn get_transactions(&self, tx_hashes: Vec<H256>) -> Result<Vec<Option<LedgerTransaction>>> {
        self.tx_store.multiple_get(tx_hashes)
    }

    pub fn get_tx_hashes(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>> {
        self.tx_sequence_info_mapping_store.multiple_get(tx_orders)
    }

    pub fn get_tx_by_order(&self, tx_order: u64) -> Result<Option<LedgerTransaction>> {
        let tx_hash = self.tx_sequence_info_mapping_store.kv_get(tx_order)?;
        match tx_hash {
            Some(tx_hash) => self.tx_store.kv_get(tx_hash),
            None => Ok(None),
        }
    }

    pub fn get_tx_hash(&self, tx_order: u64) -> Result<Option<H256>> {
        self.tx_sequence_info_mapping_store.kv_get(tx_order)
    }
}
