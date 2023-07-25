// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use raw_store::CodecKVStore;
use rooch_types::transaction::{
    AbstractTransaction, TransactionSequenceInfo, TransactionSequenceMapping, TypedTransaction,
};
use rooch_types::H256;

use crate::{
    SEQ_TRANSACTION_PREFIX_NAME, TX_SEQ_MAPPING_PREFIX_NAME, TYPED_TRANSACTION_PREFIX_NAME,
};
use raw_store::{derive_store, StoreInstance};

derive_store!(
    TypedTransactionStore,
    H256,
    TypedTransaction,
    TYPED_TRANSACTION_PREFIX_NAME
);

derive_store!(
    SeqTransactionStore,
    u128,
    TransactionSequenceInfo,
    SEQ_TRANSACTION_PREFIX_NAME
);

derive_store!(TxSeqMappingStore, u128, H256, TX_SEQ_MAPPING_PREFIX_NAME);

pub trait TransactionStore {
    fn save_transaction(&mut self, transaction: TypedTransaction) -> Result<()>;
    fn get_tx_by_hash(&self, hash: H256) -> Result<Option<TypedTransaction>>;
    fn get_tx_by_index(&self, start: u64, limit: u64) -> Result<Vec<TypedTransaction>>;

    fn save_tx_seq_info(&self, tx_seq_info: TransactionSequenceInfo) -> Result<()>;
    fn get_tx_seq_infos_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceInfo>>;
    fn save_tx_seq_info_mapping(&self, tx_order: u128, tx_hash: H256) -> Result<()>;
    fn get_tx_seq_mapping_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceMapping>>;
}

#[derive(Clone)]
pub struct TransactionDBStore {
    typed_tx_store: TypedTransactionStore,
    seq_tx_store: SeqTransactionStore,
    tx_seq_mapping: TxSeqMappingStore,
}

impl TransactionDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        TransactionDBStore {
            typed_tx_store: TypedTransactionStore::new(instance.clone()),
            seq_tx_store: SeqTransactionStore::new(instance.clone()),
            tx_seq_mapping: TxSeqMappingStore::new(instance),
        }
    }

    pub fn save_transaction(&mut self, transaction: TypedTransaction) -> Result<()> {
        self.typed_tx_store
            .kv_put(transaction.tx_hash(), transaction)
    }

    pub fn get_tx_by_hash(&self, hash: H256) -> Result<Option<TypedTransaction>> {
        self.typed_tx_store.kv_get(hash)
    }

    //TODO implements get type tx by index
    pub fn get_tx_by_index(&self, _cursor: u64, _limit: u64) -> Result<Vec<TypedTransaction>> {
        Ok(vec![])
    }

    pub fn save_tx_seq_info(&self, tx_seq_info: TransactionSequenceInfo) -> Result<()> {
        self.seq_tx_store.kv_put(tx_seq_info.tx_order, tx_seq_info)
    }

    pub fn get_tx_seq_infos_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceInfo>> {
        //  will not cross the boundary even if the size exceeds the storage capacity,
        let start = cursor.unwrap_or(0);
        let end = start + (limit as u128);
        let mut iter = self.seq_tx_store.iter()?;
        iter.seek(bcs::to_bytes(&start)?).map_err(|e| anyhow::anyhow!("Rooch TransactionStore get_tx_seq_infos_by_tx_order seek: {:?}", e))?;

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

    pub fn save_tx_seq_info_mapping(&self, tx_order: u128, tx_hash: H256) -> Result<()> {
        self.tx_seq_mapping.kv_put(tx_order, tx_hash)
    }

    pub fn get_tx_seq_mapping_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceMapping>> {
        //  will not cross the boundary even if the size exceeds the storage capacity,
        let start = cursor.unwrap_or(0);
        let end = start + (limit as u128);
        let mut iter = self.tx_seq_mapping.iter()?;
        iter.seek(bcs::to_bytes(&start)?).map_err(|e| anyhow::anyhow!("Rooch TransactionStore get_tx_seq_mapping_by_tx_order seek: {:?}", e))?;

        let data: Vec<TransactionSequenceMapping> = iter
            .filter_map(|item| {
                let (tx_order, tx_hash) =
                    item.unwrap_or_else(|_| panic!("Get item from store shoule hava a value."));
                if Option::is_some(&cursor) {
                    if tx_order > start && tx_order <= end {
                        return Some(TransactionSequenceMapping::new(tx_order, tx_hash));
                    }
                } else if tx_order >= start && tx_order < end {
                    return Some(TransactionSequenceMapping::new(tx_order, tx_hash));
                }
                None
            })
            .collect::<Vec<_>>();
        Ok(data)
    }
}
