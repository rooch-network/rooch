// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_types::h256::H256;
use raw_store::CodecKVStore;
use rooch_types::transaction::{
    AbstractTransaction, TransactionSequenceInfo, TransactionSequenceInfoMapping, TypedTransaction,
};

use crate::{
    TX_SEQUENCE_INFO_MAPPING_PREFIX_NAME, TX_SEQUENCE_INFO_PREFIX_NAME,
    TX_SEQUENCE_INFO_REVERSE_MAPPING_PREFIX_NAME, TYPED_TRANSACTION_PREFIX_NAME,
};
use raw_store::{derive_store, StoreInstance};

derive_store!(
    TypedTransactionStore,
    H256,
    TypedTransaction,
    TYPED_TRANSACTION_PREFIX_NAME
);

derive_store!(
    TxSequenceInfoStore,
    u64,
    TransactionSequenceInfo,
    TX_SEQUENCE_INFO_PREFIX_NAME
);

derive_store!(
    TxSequenceInfoMappingStore,
    u64,
    H256,
    TX_SEQUENCE_INFO_MAPPING_PREFIX_NAME
);

derive_store!(
    TxSequenceInfoReverseMappingStore,
    H256,
    u64,
    TX_SEQUENCE_INFO_REVERSE_MAPPING_PREFIX_NAME
);

pub trait TransactionStore {
    fn save_transaction(&mut self, transaction: TypedTransaction) -> Result<()>;
    fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<TypedTransaction>>;
    fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TypedTransaction>>>;

    fn save_tx_sequence_info(&self, tx_sequence_info: TransactionSequenceInfo) -> Result<()>;
    fn get_tx_sequence_infos_by_order(
        &self,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Option<TransactionSequenceInfo>>>;
    fn save_tx_sequence_info_mapping(&self, tx_order: u64, tx_hash: H256) -> Result<()>;
    fn get_tx_sequence_info_mapping_by_order(
        &self,
        tx_orders: Vec<u64>,
    ) -> Result<Vec<Option<TransactionSequenceInfoMapping>>>;

    fn save_tx_sequence_info_reverse_mapping(&self, tx_hash: H256, tx_order: u64) -> Result<()>;
    fn multi_get_tx_sequence_info_mapping_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionSequenceInfoMapping>>>;
}

#[derive(Clone)]
pub struct TransactionDBStore {
    tx_store: TypedTransactionStore,
    tx_sequence_info_store: TxSequenceInfoStore,
    tx_sequence_info_mapping_store: TxSequenceInfoMappingStore,
    tx_sequence_info_reverse_mapping_store: TxSequenceInfoReverseMappingStore,
}

impl TransactionDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        TransactionDBStore {
            tx_store: TypedTransactionStore::new(instance.clone()),
            tx_sequence_info_store: TxSequenceInfoStore::new(instance.clone()),
            tx_sequence_info_mapping_store: TxSequenceInfoMappingStore::new(instance.clone()),
            tx_sequence_info_reverse_mapping_store: TxSequenceInfoReverseMappingStore::new(
                instance,
            ),
        }
    }

    pub fn save_transaction(&mut self, transaction: TypedTransaction) -> Result<()> {
        self.tx_store.kv_put(transaction.tx_hash(), transaction)
    }

    pub fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<TypedTransaction>> {
        self.tx_store.kv_get(hash)
    }

    pub fn get_transactions(&self, tx_hashes: Vec<H256>) -> Result<Vec<Option<TypedTransaction>>> {
        self.tx_store.multiple_get(tx_hashes)
    }

    pub fn save_tx_sequence_info(&self, tx_sequence_info: TransactionSequenceInfo) -> Result<()> {
        self.tx_sequence_info_store
            .kv_put(tx_sequence_info.tx_order, tx_sequence_info)
    }

    pub fn get_tx_sequence_infos_by_order(
        &self,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<Vec<Option<TransactionSequenceInfo>>> {
        let start = cursor.unwrap_or(0);
        let end = start + limit;

        // Since tx order is strictly incremental, traversing the SMT Tree can be optimized into a multi get query to improve query performance.
        let tx_orders: Vec<_> = if cursor.is_some() {
            ((start + 1)..=end).collect()
        } else {
            (start..end).collect()
        };
        self.tx_sequence_info_store.multiple_get(tx_orders)
    }

    pub fn save_tx_sequence_info_mapping(&self, tx_order: u64, tx_hash: H256) -> Result<()> {
        self.tx_sequence_info_mapping_store
            .kv_put(tx_order, tx_hash)
    }

    pub fn get_tx_sequence_info_mapping_by_order(
        &self,
        tx_orders: Vec<u64>,
    ) -> Result<Vec<Option<TransactionSequenceInfoMapping>>> {
        let mappings = self
            .tx_sequence_info_mapping_store
            .multiple_get(tx_orders.clone())?;

        mappings
            .into_iter()
            .enumerate()
            .map(|(index, value)| match value {
                Some(tx_hash) => {
                    let tx_order = tx_orders[index];
                    let tx_sequence_info_mapping =
                        TransactionSequenceInfoMapping { tx_order, tx_hash };
                    Ok(Some(tx_sequence_info_mapping))
                }
                None => Ok(None),
            })
            .collect()
    }

    pub fn get_tx_sequence_infos(
        &self,
        orders: Vec<u64>,
    ) -> Result<Vec<Option<TransactionSequenceInfo>>> {
        self.tx_sequence_info_store.multiple_get(orders)
    }

    pub fn save_tx_sequence_info_reverse_mapping(
        &self,
        tx_hash: H256,
        tx_order: u64,
    ) -> Result<()> {
        self.tx_sequence_info_reverse_mapping_store
            .kv_put(tx_hash, tx_order)
    }

    pub fn multi_get_tx_sequence_info_mapping_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionSequenceInfoMapping>>> {
        let mappings = self
            .tx_sequence_info_reverse_mapping_store
            .multiple_get(tx_hashes.clone())?;

        mappings
            .into_iter()
            .enumerate()
            .map(|(index, value)| match value {
                Some(tx_order) => {
                    let tx_hash = tx_hashes[index];
                    let tx_sequence_info_mapping =
                        TransactionSequenceInfoMapping { tx_order, tx_hash };
                    Ok(Some(tx_sequence_info_mapping))
                }
                None => Ok(None),
            })
            .collect()
    }
}
