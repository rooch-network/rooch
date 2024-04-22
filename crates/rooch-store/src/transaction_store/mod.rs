// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{TRANSACTION_PREFIX_NAME, TX_SEQUENCE_INFO_MAPPING_PREFIX_NAME};
use anyhow::Result;
use moveos_types::h256::H256;
use raw_store::CodecKVStore;
use raw_store::{derive_store, StoreInstance};
use rooch_types::transaction::LedgerTransaction;

const MAX_RESULT_AMOUNT: u64 = 10000000;

derive_store!(
    LedgerTransactionStore,
    H256,
    LedgerTransaction,
    TRANSACTION_PREFIX_NAME
);

derive_store!(
    TxSequenceInfoMappingStore,
    u64,
    H256,
    TX_SEQUENCE_INFO_MAPPING_PREFIX_NAME
);
pub trait TransactionStore {
    fn save_transaction(&mut self, transaction: LedgerTransaction) -> Result<()>;
    fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>>;
    fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<LedgerTransaction>>>;

    fn get_tx_hashs(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>>;

    fn get_tx_hashs_by_order(&self, cursor: Option<u64>, limit: u64) -> Result<Vec<Option<H256>>> {
        let start = cursor.unwrap_or(0);
        let end = start + limit;

        let gaps = match end.checked_sub(start) {
            None => return Err(anyhow::Error::msg("end value is overflow")),
            Some(v) => v,
        };
        if gaps > MAX_RESULT_AMOUNT {
            return Err(anyhow::Error::msg("end value is overflow"));
        }

        // Since tx order is strictly incremental, traversing the SMT Tree can be optimized into a multi get query to improve query performance.
        let tx_orders: Vec<_> = if cursor.is_some() {
            ((start + 1)..=end).collect()
        } else {
            (start..end).collect()
        };
        self.get_tx_hashs(tx_orders)
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

    pub fn save_transaction(&mut self, mut transaction: LedgerTransaction) -> Result<()> {
        let tx_hash = transaction.tx_hash();
        let tx_order = transaction.sequence_info.tx_order;
        self.tx_store.kv_put(tx_hash, transaction)?;
        self.tx_sequence_info_mapping_store
            .kv_put(tx_order, tx_hash)
    }

    pub fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>> {
        self.tx_store.kv_get(hash)
    }

    pub fn get_transactions(&self, tx_hashes: Vec<H256>) -> Result<Vec<Option<LedgerTransaction>>> {
        self.tx_store.multiple_get(tx_hashes)
    }

    pub fn get_tx_hashs(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>> {
        self.tx_sequence_info_mapping_store.multiple_get(tx_orders)
    }
}
