// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::transaction_store::{TransactionDBStore, TransactionStore};
use anyhow::Result;
use moveos_config::store_config::RocksdbConfig;
use moveos_config::temp_dir;
use once_cell::sync::Lazy;
use raw_store::rocks::RocksDB;
use raw_store::{ColumnFamilyName, StoreInstance};
use rooch_types::transaction::{
    TransactionSequenceInfo, TransactionSequenceMapping, TypedTransaction,
};
use rooch_types::H256;
use std::fmt::{Debug, Display, Formatter};

pub mod transaction_store;

// pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
pub const TYPED_TRANSACTION_PREFIX_NAME: ColumnFamilyName = "typed_transaction";
pub const SEQ_TRANSACTION_PREFIX_NAME: ColumnFamilyName = "seq_transaction";
pub const TX_SEQ_MAPPING_PREFIX_NAME: ColumnFamilyName = "tx_seq_mapping";

///db store use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
static VEC_PREFIX_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        TYPED_TRANSACTION_PREFIX_NAME,
        SEQ_TRANSACTION_PREFIX_NAME,
        TX_SEQ_MAPPING_PREFIX_NAME,
    ]
});

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StoreMeta {}

impl StoreMeta {
    pub fn get_column_family_names() -> &'static [ColumnFamilyName] {
        &VEC_PREFIX_NAME
    }
}

#[derive(Clone)]
pub struct RoochStore {
    pub transaction_store: TransactionDBStore,
}

impl RoochStore {
    pub fn new(instance: StoreInstance) -> Result<Self> {
        let store = Self {
            transaction_store: TransactionDBStore::new(instance),
        };
        Ok(store)
    }

    //TODO implement a memory mock store
    pub fn mock_rooch_store() -> Self {
        Self::new(StoreInstance::new_db_instance(
            RocksDB::new(
                temp_dir().path(),
                moveos_store::StoreMeta::get_column_family_names().to_vec(),
                RocksdbConfig::default(),
                None,
            )
            .expect("init db error"),
        ))
        .expect("init rooch store error")
    }

    pub fn get_transaction_store(&self) -> &TransactionDBStore {
        &self.transaction_store
    }
}

impl Display for RoochStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clone())
    }
}
impl Debug for RoochStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl TransactionStore for RoochStore {
    fn save_transaction(&mut self, transaction: TypedTransaction) -> Result<()> {
        self.transaction_store.save_transaction(transaction)
    }

    fn get_tx_by_hash(&self, hash: H256) -> Result<Option<TypedTransaction>> {
        self.transaction_store.get_tx_by_hash(hash)
    }

    fn get_tx_by_index(&self, start: u64, limit: u64) -> Result<Vec<TypedTransaction>> {
        self.transaction_store.get_tx_by_index(start, limit)
    }

    fn save_tx_seq_info(&self, tx_seq_info: TransactionSequenceInfo) -> Result<()> {
        self.transaction_store.save_tx_seq_info(tx_seq_info)
    }

    fn get_tx_seq_infos_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceInfo>> {
        self.transaction_store
            .get_tx_seq_infos_by_tx_order(cursor, limit)
    }

    fn save_tx_seq_info_mapping(&self, tx_order: u128, tx_hash: H256) -> Result<()> {
        self.transaction_store
            .save_tx_seq_info_mapping(tx_order, tx_hash)
    }

    fn get_tx_seq_mapping_by_tx_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceMapping>> {
        self.transaction_store
            .get_tx_seq_mapping_by_tx_order(cursor, limit)
    }
}
