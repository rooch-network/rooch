// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use raw_store::CodecKVStore;
use rooch_types::transaction::{
    AbstractTransaction, TransactionSequenceInfo, TransactionSequenceInfoMapping, TypedTransaction,
};
use rooch_types::H256;

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
    u128,
    TransactionSequenceInfo,
    TX_SEQUENCE_INFO_PREFIX_NAME
);

derive_store!(
    TxSequenceInfoMappingStore,
    u128,
    H256,
    TX_SEQUENCE_INFO_MAPPING_PREFIX_NAME
);

derive_store!(
    TxSequenceInfoReverseMappingStore,
    H256,
    u128,
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
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceInfo>>;
    fn save_tx_sequence_info_mapping(&self, tx_order: u128, tx_hash: H256) -> Result<()>;
    fn get_tx_sequence_info_mapping_by_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceInfoMapping>>;

    fn save_tx_sequence_info_reverse_mapping(&self, tx_hash: H256, tx_order: u128) -> Result<()>;
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
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceInfo>> {
        //  will not cross the boundary even if the size exceeds the storage capacity,
        let start = cursor.unwrap_or(0);
        let end = start + (limit as u128);
        let mut iter = self.tx_sequence_info_store.iter()?;
        iter.seek(bcs::to_bytes(&start)?).map_err(|e| {
            anyhow::anyhow!(
                "Rooch TransactionStore get_tx_sequence_infos_by_order seek: {:?}",
                e
            )
        })?;

        let data: Vec<TransactionSequenceInfo> = iter
            .filter_map(|item| {
                let (tx_order, seq_info) =
                    item.unwrap_or_else(|_| panic!("Get item from store shoule hava a value."));
                if Option::is_some(&cursor) {
                    if tx_order > start && tx_order <= end {
                        return Some(seq_info);
                    }
                } else if tx_order >= start && tx_order < end {
                    return Some(seq_info);
                }
                None
            })
            .collect::<Vec<_>>();
        Ok(data)
    }

    pub fn save_tx_sequence_info_mapping(&self, tx_order: u128, tx_hash: H256) -> Result<()> {
        self.tx_sequence_info_mapping_store
            .kv_put(tx_order, tx_hash)
    }

    pub fn get_tx_sequence_mapping_by_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceInfoMapping>> {
        //  will not cross the boundary even if the size exceeds the storage capacity,
        let start = cursor.unwrap_or(0);
        let end = start + (limit as u128);
        let mut iter = self.tx_sequence_info_mapping_store.iter()?;
        iter.seek(bcs::to_bytes(&start)?).map_err(|e| {
            anyhow::anyhow!(
                "Rooch TransactionStore get_tx_sequence_mapping_by_order seek: {:?}",
                e
            )
        })?;

        let data: Vec<TransactionSequenceInfoMapping> = iter
            .filter_map(|item| {
                let (tx_order, tx_hash) =
                    item.unwrap_or_else(|_| panic!("Get item from store shoule hava a value."));
                if Option::is_some(&cursor) {
                    if tx_order > start && tx_order <= end {
                        return Some(TransactionSequenceInfoMapping::new(tx_order, tx_hash));
                    }
                } else if tx_order >= start && tx_order < end {
                    return Some(TransactionSequenceInfoMapping::new(tx_order, tx_hash));
                }
                None
            })
            .collect::<Vec<_>>();
        Ok(data)
    }

    pub fn get_tx_sequence_infos(
        &self,
        orders: Vec<u128>,
    ) -> Result<Vec<Option<TransactionSequenceInfo>>> {
        self.tx_sequence_info_store.multiple_get(orders)
    }

    pub fn save_tx_sequence_info_reverse_mapping(
        &self,
        tx_hash: H256,
        tx_order: u128,
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

        // let mut result: Vec<Option<TransactionSequenceInfoMapping>> = vec![];
        // for (index, tx_hash) in tx_hashes.iter().enumerate() {
        //     let tx_order = mappings[index].ok_or(anyhow::anyhow!("Invalid transaction sequence info mapping"))?;
        //     let tx_sequence_info_mapping = TransactionSequenceInfoMapping {
        //         // pub tx_order: u128,
        //         // /// The tx hash.
        //         // pub tx_hash: H256,
        //         tx_order,
        //         *tx_hash,
        //     };
        //     result.push(tx_sequence_info_mapping);
        // }

        // Ok(result)
    }
}
