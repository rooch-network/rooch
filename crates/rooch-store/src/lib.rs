// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::meta_store::{MetaDBStore, MetaStore};
use crate::transaction_store::{TransactionDBStore, TransactionStore};
use anyhow::Result;
use moveos_types::h256::H256;
use once_cell::sync::Lazy;
use raw_store::{ColumnFamilyName, StoreInstance};
use rooch_types::sequencer::SequencerOrder;
use rooch_types::transaction::LedgerTransaction;
use std::fmt::{Debug, Display, Formatter};

pub mod meta_store;
pub mod transaction_store;

// pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
pub const TRANSACTION_PREFIX_NAME: ColumnFamilyName = "transaction";
pub const TX_SEQUENCE_INFO_MAPPING_PREFIX_NAME: ColumnFamilyName = "tx_sequence_info_mapping";

pub const META_SEQUENCER_ORDER_PREFIX_NAME: ColumnFamilyName = "meta_sequencer_order";

///db store use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
static VEC_PREFIX_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        TRANSACTION_PREFIX_NAME,
        TX_SEQUENCE_INFO_MAPPING_PREFIX_NAME,
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
    fn save_transaction(&self, tx: LedgerTransaction) -> Result<()> {
        let sequencer_order = SequencerOrder::new(tx.sequence_info.tx_order);
        self.transaction_store.save_transaction(tx)?;
        self.meta_store.save_sequencer_order(sequencer_order)
    }

    fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>> {
        self.transaction_store.get_transaction_by_hash(hash)
    }

    fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<LedgerTransaction>>> {
        self.transaction_store.get_transactions(tx_hashes)
    }

    fn get_tx_hashs(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>> {
        self.transaction_store.get_tx_hashs(tx_orders)
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
