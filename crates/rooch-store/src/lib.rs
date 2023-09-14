// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::meta_store::{MetaDBStore, MetaStore};
use crate::transaction_store::{TransactionDBStore, TransactionStore};
use anyhow::Result;
use moveos_config::store_config::RocksdbConfig;
use moveos_config::temp_dir;
use once_cell::sync::Lazy;
use raw_store::rocks::RocksDB;
use raw_store::{ColumnFamilyName, StoreInstance};
use rooch_types::sequencer::SequencerOrder;
use rooch_types::transaction::{
    TransactionSequenceInfo, TransactionSequenceMapping, TypedTransaction,
};
use rooch_types::H256;
use std::fmt::{Debug, Display, Formatter};

pub mod meta_store;
pub mod transaction_store;

// pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
pub const TYPED_TRANSACTION_PREFIX_NAME: ColumnFamilyName = "typed_transaction";
pub const SEQ_TRANSACTION_PREFIX_NAME: ColumnFamilyName = "seq_transaction";
pub const TX_SEQ_MAPPING_PREFIX_NAME: ColumnFamilyName = "tx_seq_mapping";

pub const META_SEQUENCER_ORDER_PREFIX_NAME: ColumnFamilyName = "meta_sequencer_order";

///db store use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
static VEC_PREFIX_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        TYPED_TRANSACTION_PREFIX_NAME,
        SEQ_TRANSACTION_PREFIX_NAME,
        TX_SEQ_MAPPING_PREFIX_NAME,
        META_SEQUENCER_ORDER_PREFIX_NAME,
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
    pub meta_store: MetaDBStore,
}

impl RoochStore {
    pub fn new(instance: StoreInstance) -> Result<Self> {
        let store = Self {
            transaction_store: TransactionDBStore::new(instance.clone()),
            meta_store: MetaDBStore::new(instance),
        };
        Ok(store)
    }

    //TODO implement a memory mock store
    pub fn mock_rooch_store() -> Result<Self> {
        Self::new(StoreInstance::new_db_instance(RocksDB::new(
            temp_dir().path(),
            moveos_store::StoreMeta::get_column_family_names().to_vec(),
            RocksdbConfig::default(),
            None,
        )?))
    }

    pub fn get_transaction_store(&self) -> &TransactionDBStore {
        &self.transaction_store
    }

    pub fn get_meta_store(&self) -> &MetaDBStore {
        &self.meta_store
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

    fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<TypedTransaction>> {
        self.transaction_store.get_transaction_by_hash(hash)
    }

    fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TypedTransaction>>> {
        self.transaction_store.get_transactions(tx_hashes)
    }

    fn save_tx_seq_info(&self, tx_seq_info: TransactionSequenceInfo) -> Result<()> {
        self.transaction_store.save_tx_seq_info(tx_seq_info)
    }

    fn get_tx_seq_infos_by_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceInfo>> {
        self.transaction_store
            .get_tx_seq_infos_by_order(cursor, limit)
    }

    fn save_tx_seq_info_mapping(&self, tx_order: u128, tx_hash: H256) -> Result<()> {
        self.transaction_store
            .save_tx_seq_info_mapping(tx_order, tx_hash)
    }

    fn get_tx_seq_mapping_by_order(
        &self,
        cursor: Option<u128>,
        limit: u64,
    ) -> Result<Vec<TransactionSequenceMapping>> {
        self.transaction_store
            .get_tx_seq_mapping_by_order(cursor, limit)
    }
}

impl MetaStore for RoochStore {
    fn get_sequencer_order(&self) -> Result<Option<SequencerOrder>> {
        self.get_meta_store().get_sequencer_order()
    }

    fn save_sequencer_order(&self, sequencer_order: SequencerOrder) -> Result<()> {
        self.get_meta_store().save_sequencer_order(sequencer_order)
    }
}
