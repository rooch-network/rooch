// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator_store::{AccumulatorStore, TransactionAccumulatorStore};
use crate::da_store::{DAMetaDBStore, DAMetaStore};
use crate::meta_store::{MetaDBStore, MetaStore, SEQUENCER_INFO_KEY};
use crate::state_store::{StateDBStore, StateStore};
use crate::transaction_store::{TransactionDBStore, TransactionStore};
use accumulator::{AccumulatorNode, AccumulatorTreeStore};
use anyhow::Result;
use moveos_common::utils::to_bytes;
use moveos_config::store_config::RocksdbConfig;
use moveos_config::DataDirPath;
use moveos_types::h256::H256;
use moveos_types::state::StateChangeSetExt;
use once_cell::sync::Lazy;
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
use raw_store::rocks::batch::WriteBatch;
use raw_store::rocks::RocksDB;
use raw_store::traits::DBStore;
use raw_store::{ColumnFamilyName, StoreInstance};
use rooch_types::da::batch::{BlockRange, BlockSubmitState};
use rooch_types::sequencer::SequencerInfo;
use rooch_types::transaction::LedgerTransaction;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};

pub mod accumulator_store;
pub mod da_store;
pub mod meta_store;
pub mod state_store;
pub mod transaction_store;

#[cfg(test)]
mod tests;

// pub const DEFAULT_COLUMN_FAMILY_NAME: ColumnFamilyName = "default";
pub const TRANSACTION_COLUMN_FAMILY_NAME: ColumnFamilyName = "transaction";
pub const TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME: ColumnFamilyName =
    "tx_sequence_info_mapping";
pub const META_SEQUENCER_INFO_COLUMN_FAMILY_NAME: ColumnFamilyName = "meta_sequencer_info";
pub const TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME: ColumnFamilyName = "transaction_acc_node";

pub const STATE_CHANGE_SET_COLUMN_FAMILY_NAME: ColumnFamilyName = "state_change_set";

pub const DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME: ColumnFamilyName = "da_block_submit_state";
pub const DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME: ColumnFamilyName = "da_last_block_number";

///db store use cf_name vec to init
/// Please note that adding a column family needs to be added in vec simultaneously, remember！！
static VEC_COLUMN_FAMILY_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        TRANSACTION_COLUMN_FAMILY_NAME,
        TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
        META_SEQUENCER_INFO_COLUMN_FAMILY_NAME,
        TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME,
        STATE_CHANGE_SET_COLUMN_FAMILY_NAME,
        DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME,
        DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME,
    ]
});

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StoreMeta {}

impl StoreMeta {
    pub fn get_column_family_names() -> &'static [ColumnFamilyName] {
        &VEC_COLUMN_FAMILY_NAME
    }
}

#[derive(Clone)]
pub struct RoochStore {
    pub store_instance: StoreInstance,
    pub transaction_store: TransactionDBStore,
    pub meta_store: MetaDBStore,
    pub transaction_accumulator_store: AccumulatorStore<TransactionAccumulatorStore>,
    pub state_store: StateDBStore,
    pub da_meta_store: DAMetaDBStore,
}

impl RoochStore {
    pub fn new(db_path: &Path, registry: &Registry) -> Result<Self> {
        let db_metrics = DBMetrics::get_or_init(registry).clone();
        let instance = StoreInstance::new_db_instance(
            RocksDB::new(
                db_path,
                StoreMeta::get_column_family_names().to_vec(),
                RocksdbConfig::default(),
            )?,
            db_metrics,
        );
        Self::new_with_instance(instance, registry)
    }

    pub fn new_with_instance(instance: StoreInstance, _registry: &Registry) -> Result<Self> {
        let da_meta_store = DAMetaDBStore::new(instance.clone())?;
        let store = Self {
            store_instance: instance.clone(),
            transaction_store: TransactionDBStore::new(instance.clone()),
            meta_store: MetaDBStore::new(instance.clone()),
            transaction_accumulator_store: AccumulatorStore::new_transaction_accumulator_store(
                instance.clone(),
            ),
            state_store: StateDBStore::new(instance.clone()),
            da_meta_store,
        };
        Ok(store)
    }

    pub fn mock_rooch_store() -> Result<(Self, DataDirPath)> {
        let tmpdir = moveos_config::temp_dir();
        let registry = prometheus::Registry::new();

        //The testcases should hold the tmpdir to prevent the tmpdir from being deleted.
        Ok((Self::new(tmpdir.path(), &registry)?, tmpdir))
    }

    pub fn get_transaction_store(&self) -> &TransactionDBStore {
        &self.transaction_store
    }

    pub fn get_meta_store(&self) -> &MetaDBStore {
        &self.meta_store
    }

    pub fn get_transaction_accumulator_store(&self) -> Arc<dyn AccumulatorTreeStore> {
        Arc::new(self.transaction_accumulator_store.clone())
    }

    pub fn get_state_store(&self) -> &StateDBStore {
        &self.state_store
    }

    pub fn get_da_meta_store(&self) -> &DAMetaDBStore {
        &self.da_meta_store
    }

    /// atomic save updates made by Sequencer.sequence(tx) to the store
    pub fn save_sequenced_tx(
        &self,
        tx_hash: H256,
        tx: LedgerTransaction,
        sequencer_info: SequencerInfo,
        accumulator_nodes: Option<Vec<AccumulatorNode>>,
    ) -> Result<()> {
        // TODO use txn GetForUpdate to guard against Read-Write Conflicts (need open rocksdb with TransactionDB)
        let pre_sequencer_info = self.get_sequencer_info()?;
        if let Some(pre_sequencer_info) = pre_sequencer_info {
            if sequencer_info.last_order != pre_sequencer_info.last_order + 1 {
                return Err(anyhow::anyhow!("Sequencer order is not continuous"));
            }
        }

        let inner_store = &self.store_instance;
        let tx_order = tx.sequence_info.tx_order;
        let mut write_batch = WriteBatch::new();
        let mut cf_names = vec![
            TRANSACTION_COLUMN_FAMILY_NAME,
            TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
            META_SEQUENCER_INFO_COLUMN_FAMILY_NAME,
        ];
        write_batch.put(to_bytes(&tx_hash).unwrap(), to_bytes(&tx).unwrap())?;
        write_batch.put(to_bytes(&tx_order).unwrap(), to_bytes(&tx_hash).unwrap())?;
        write_batch.put(
            to_bytes(SEQUENCER_INFO_KEY).unwrap(),
            to_bytes(&sequencer_info).unwrap(),
        )?;
        if let Some(accumulator_nodes) = accumulator_nodes {
            for node in accumulator_nodes {
                write_batch.put(to_bytes(&node.hash()).unwrap(), to_bytes(&node).unwrap())?;
                cf_names.push(TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME);
            }
        }
        inner_store.write_batch_across_cfs(cf_names, write_batch, true)?;
        Ok(())
    }

    pub fn repair(&self, thorough: bool, _exec: bool) -> Result<(usize, usize)> {
        let sequence_info = self
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("Sequencer info not found"))?;
        let exp_last_order = sequence_info.last_order;
        if exp_last_order == 0 {
            return Ok((0, 0));
        }

        let mut issues = 0;
        let mut fixed = 0;

        if thorough {
            issues += self.check_thorough(exp_last_order)?;
            info!("Thorough check done, issues: {}", issues);
        }

        // exec or not, DA will exec repair automatically because there won't be external dependencies for repair DA meta
        let (da_issues, da_fixed) = self
            .da_meta_store
            .try_repair_da_meta(exp_last_order, thorough)?;
        info!("DA repair done, issues: {}, fixed: {}", da_issues, da_fixed);
        issues += da_issues;
        fixed += da_fixed;

        Ok((issues, fixed))
    }

    // check sequenced tx updates are atomic or not
    // after 0.7.6 released and fixed historical data, this check can be removed
    fn check_thorough(&self, exp_last_order: u64) -> Result<usize> {
        let mut issues = 0;
        let mut lost_tx_hashes = vec![];
        let mut lost_tx = vec![];

        for tx_order in 0..=exp_last_order {
            let tx_hash_opt = self
                .transaction_store
                .get_tx_hashes(vec![tx_order])?
                .pop()
                .flatten();
            match tx_hash_opt {
                Some(tx_hash) => {
                    let tx = self.transaction_store.get_transaction_by_hash(tx_hash)?;
                    if tx.is_none() {
                        lost_tx.push(tx_hash);
                        issues += 1;
                    }
                }
                None => {
                    lost_tx_hashes.push(tx_order);
                    issues += 1;
                }
            }
        }

        if !lost_tx_hashes.is_empty() {
            error!(
                "Lost tx hashes({}): {:?}",
                lost_tx_hashes.len(),
                lost_tx_hashes
            );
        }
        if !lost_tx.is_empty() {
            error!("Lost txs({}): {:?}", lost_tx.len(), lost_tx);
        }

        Ok(issues)
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
    fn remove_transaction(&self, tx_hash: H256, tx_order: u64) -> Result<()> {
        self.transaction_store.remove_transaction(tx_hash, tx_order)
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

    fn get_tx_hashes(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>> {
        self.transaction_store.get_tx_hashes(tx_orders)
    }
}

impl MetaStore for RoochStore {
    fn get_sequencer_info(&self) -> Result<Option<SequencerInfo>> {
        self.get_meta_store().get_sequencer_info()
    }

    fn save_sequencer_info(&self, sequencer_info: SequencerInfo) -> Result<()> {
        self.get_meta_store().save_sequencer_info(sequencer_info)
    }

    fn remove_sequencer_info(&self) -> Result<()> {
        self.get_meta_store().remove_sequence_info()
    }
}

impl StateStore for RoochStore {
    // Setting TTL directly in RocksDB may not be a good choice.
    // RocksDB uses compaction to remove expired keys,
    // and it may also have performance impact.
    // TODO Cleaning up data regularly may be an option
    fn save_state_change_set(
        &self,
        tx_order: u64,
        state_change_set: StateChangeSetExt,
    ) -> Result<()> {
        self.get_state_store()
            .save_state_change_set(tx_order, state_change_set)
    }

    fn get_state_change_set(&self, tx_order: u64) -> Result<Option<StateChangeSetExt>> {
        self.get_state_store().get_state_change_set(tx_order)
    }

    fn multi_get_state_change_set(
        &self,
        tx_orders: Vec<u64>,
    ) -> Result<Vec<Option<StateChangeSetExt>>> {
        self.get_state_store().multi_get_state_change_set(tx_orders)
    }

    fn remove_state_change_set(&self, tx_order: u64) -> Result<()> {
        self.get_state_store().remove_state_change_set(tx_order)
    }
}

impl DAMetaStore for RoochStore {
    fn try_repair_da_meta(&self, last_order: u64, thorough: bool) -> Result<(usize, usize)> {
        self.get_da_meta_store()
            .try_repair_da_meta(last_order, thorough)
    }

    fn append_submitting_block(&self, tx_order_start: u64, tx_order_end: u64) -> Result<u128> {
        self.get_da_meta_store()
            .append_submitting_block(tx_order_start, tx_order_end)
    }

    fn get_submitting_blocks(
        &self,
        start_block: u128,
        exp_count: Option<usize>,
    ) -> Result<Vec<BlockRange>> {
        self.get_da_meta_store()
            .get_submitting_blocks(start_block, exp_count)
    }

    fn set_submitting_block_done(
        &self,
        block_number: u128,
        tx_order_start: u64,
        tx_order_end: u64,
        bash_hash: H256,
    ) -> Result<()> {
        self.get_da_meta_store().set_submitting_block_done(
            block_number,
            tx_order_start,
            tx_order_end,
            bash_hash,
        )
    }

    fn set_background_submit_block_cursor(&self, cursor: u128) -> Result<()> {
        self.get_da_meta_store()
            .set_background_submit_block_cursor(cursor)
    }

    fn get_background_submit_block_cursor(&self) -> Result<Option<u128>> {
        self.get_da_meta_store()
            .get_background_submit_block_cursor()
    }

    fn get_last_block_number(&self) -> Result<Option<u128>> {
        self.get_da_meta_store().get_last_block_number()
    }

    fn get_block_state(&self, block_number: u128) -> Result<BlockSubmitState> {
        self.get_da_meta_store().get_block_state(block_number)
    }
}
