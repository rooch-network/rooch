// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use accumulator::accumulator_info::AccumulatorInfo;
use anyhow::{anyhow, Error, Result};
use moveos_common::utils::to_bytes;
use moveos_store::config_store::STARTUP_INFO_KEY;
use moveos_store::transaction_store::TransactionStore as TxExecutionInfoStore;
use moveos_store::{
    MoveOSStore, CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME,
    TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME,
};
use moveos_types::access_path::AccessPath;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::state::StateChangeSetExt;
use moveos_types::state_resolver::{RootObjectResolver, StateReader};
use moveos_types::transaction::TransactionExecutionInfo;
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
use raw_store::rocks::batch::WriteBatch;
use raw_store::traits::DBStore;
use raw_store::{rocks::RocksDB, StoreInstance};
use rooch_config::store_config::StoreConfig;
use rooch_indexer::store::traits::IndexerStoreTrait;
use rooch_indexer::{indexer_reader::IndexerReader, IndexerStore};
use rooch_store::meta_store::{MetaStore, SEQUENCER_INFO_KEY};
use rooch_store::state_store::StateStore;
use rooch_store::transaction_store::TransactionStore;
use rooch_store::{
    RoochStore, META_SEQUENCER_INFO_COLUMN_FAMILY_NAME, STATE_CHANGE_SET_COLUMN_FAMILY_NAME,
    TRANSACTION_COLUMN_FAMILY_NAME, TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
};
use rooch_types::finality_block::L2BlockRef;
use rooch_types::indexer::state::{
    collect_revert_object_change_ids, handle_revert_object_change, IndexerObjectStateChangeSet,
    IndexerObjectStatesIndexGenerator,
};
use rooch_types::sequencer::SequencerInfo;
use tracing::error;

#[derive(Clone)]
pub struct RoochDB {
    pub moveos_store: MoveOSStore,
    pub rooch_store: RoochStore,
    pub indexer_store: IndexerStore,
    pub indexer_reader: IndexerReader,
}

impl RoochDB {
    pub fn init(config: &StoreConfig, registry: &Registry) -> Result<Self> {
        let instance = Self::generate_store_instance(config, registry)?;
        Self::init_with_instance(config, instance, registry)
    }

    pub fn init_with_instance(
        config: &StoreConfig,
        instance: StoreInstance,
        registry: &Registry,
    ) -> Result<Self> {
        let indexer_dir = config.get_indexer_dir();
        let moveos_store = MoveOSStore::new_with_instance(instance.clone(), registry)?;
        let rooch_store = RoochStore::new_with_instance(instance.clone(), registry)?;
        let indexer_store = IndexerStore::new(indexer_dir.clone(), registry)?;
        let indexer_reader = IndexerReader::new(indexer_dir, registry)?;

        Ok(Self {
            moveos_store,
            rooch_store,
            indexer_store,
            indexer_reader,
        })
    }

    pub fn generate_store_instance(
        config: &StoreConfig,
        registry: &Registry,
    ) -> Result<StoreInstance> {
        let store_dir = config.get_store_dir();
        let mut column_families = moveos_store::StoreMeta::get_column_family_names().to_vec();
        column_families.append(&mut rooch_store::StoreMeta::get_column_family_names().to_vec());
        //ensure no duplicate column families
        {
            let mut set = HashSet::with_capacity(column_families.len());
            column_families.iter().for_each(|cf| {
                if !set.insert(cf) {
                    panic!("Duplicate column family: {}", cf);
                }
            });
        }

        let db_metrics = DBMetrics::get_or_init(registry).clone();
        let instance = StoreInstance::new_db_instance(
            RocksDB::new(store_dir, column_families, config.rocksdb_config())?,
            db_metrics,
        );

        Ok(instance)
    }

    pub fn init_with_mock_metrics_for_test(config: &StoreConfig) -> Result<Self> {
        let registry = Registry::new();
        Self::init(config, &registry)
    }

    pub fn latest_root(&self) -> Result<Option<ObjectMeta>> {
        let startup_info = self.moveos_store.config_store.get_startup_info()?;

        Ok(startup_info.map(|s| s.into_root_metadata()))
    }

    /// revert tx with these operations:
    /// 1. check preconditions
    /// 2. remove the tx + save previous tx as startup (atomic)
    /// 3. revert indexer
    pub fn revert_tx(&self, tx_hash: H256) -> Result<()> {
        let (tx_order, previous_accumulator_info, previous_execution_info) =
            self.check_revert_tx(tx_hash)?;
        let previous_tx_hash = previous_execution_info.tx_hash;
        self.inner_revert(
            tx_order,
            tx_hash,
            Some(previous_accumulator_info),
            Some(previous_execution_info),
            true,
        )?;
        tracing::info!(
            "revert tx succeed: tx_hash: {:?}, tx_order {}, previous_tx_hash: {:?}",
            tx_hash,
            tx_order,
            previous_tx_hash,
        );
        Ok(())
    }

    /// revert tx unsafe with these operations:
    /// 1. remove the tx (atomic)
    /// 2. revert indexer
    ///
    /// warning: this method is not safe, it will not check revert preconditions or save previous tx as startup
    pub fn revert_tx_unsafe(&self, tx_order: u64, tx_hash: H256) -> Result<()> {
        self.inner_revert(tx_order, tx_hash, None, None, false)?;
        tracing::info!(
            "revert tx unsafe succeed: tx_hash: {:?}, tx_order {}",
            tx_hash,
            tx_order,
        );
        Ok(())
    }

    // check revert tx preconditions(no side-effect):
    // 1. tx existed
    // 2. tx is last tx
    // 3. previous tx existed and has execution info
    fn check_revert_tx(
        &self,
        tx_hash: H256,
    ) -> Result<(u64, AccumulatorInfo, TransactionExecutionInfo)> {
        // ensure tx existed
        let ledger_tx_opt = self
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)?;
        let sequencer_info = ledger_tx_opt
            .as_ref()
            .map(|tx| tx.sequence_info.clone())
            .ok_or_else(|| anyhow::anyhow!("revert tx failed: ledger tx not found for tx_hash {:?}. database is inconsistent", tx_hash))?;
        let tx_order = sequencer_info.tx_order;
        assert!(
            tx_order > 0,
            "revert tx failed: tx_order {} is invalid",
            tx_order
        );

        // ensure tx is last tx
        let last_sequencer_info = self
            .rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| {
                anyhow::anyhow!("Load sequencer info failed. database is inconsistent")
            })?;
        let last_tx_order = last_sequencer_info.last_order;
        assert_eq!(
            sequencer_info.tx_order, last_sequencer_info.last_order,
            "revert tx failed: tx_order {} is not last tx_order {}. tx_hash: {:?}",
            tx_order, last_tx_order, tx_hash
        );

        // ensure previous tx existed
        let previous_tx_order = tx_order - 1;
        let previous_tx_hash_opt = self
            .rooch_store
            .transaction_store
            .get_tx_hashes(vec![previous_tx_order])?;
        if previous_tx_hash_opt.is_empty() || previous_tx_hash_opt[0].is_none() {
            return Err(Error::msg(format!(
                "revert tx failed: tx_hash not found for tx_order(previous) {:?}. database is inconsistent",
                previous_tx_order
            )));
        }
        let previous_tx_hash = previous_tx_hash_opt[0].unwrap();
        let previous_ledger_tx_opt = self
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(previous_tx_hash)?;
        if previous_ledger_tx_opt.is_none() {
            return Err(Error::msg(format!(
                "revert tx failed: ledger tx(previous) not found for tx_hash {:?}. database is inconsistent",
                previous_tx_hash
            )));
        }

        let previous_execution_info_opt = self
            .moveos_store
            .transaction_store
            .get_tx_execution_info(previous_tx_hash)?;
        if previous_execution_info_opt.is_none() {
            return Err(Error::msg(format!(
                "the previous execution info not exist via tx_hash {}, revert tx failed",
                previous_tx_hash
            )));
        }

        Ok((
            tx_order,
            previous_ledger_tx_opt
                .unwrap()
                .sequence_info
                .tx_accumulator_info(),
            previous_execution_info_opt.unwrap(),
        ))
    }

    // rm tx and save previous as startup(option) + revert indexer
    fn inner_revert(
        &self,
        tx_order: u64,
        tx_hash: H256,
        previous_accumulator_info_opt: Option<AccumulatorInfo>,
        previous_execution_info_opt: Option<TransactionExecutionInfo>,
        update_startup: bool,
    ) -> Result<()> {
        let state_change_set_ext_opt = self.rooch_store.get_state_change_set(tx_order)?;

        let inner_store = &self.rooch_store.store_instance;
        let mut write_batch = WriteBatch::new();
        // remove
        write_batch.delete(to_bytes(&tx_hash)?)?; // tx_hash:tx
        write_batch.delete(to_bytes(&tx_order)?)?; // tx_order:tx_hash
        write_batch.delete(to_bytes(&tx_hash)?)?; // tx_hash:tx_execution_info
        write_batch.delete(to_bytes(&tx_order)?)?; // tx_order:tx_state_change_set
        let mut cf_names = vec![
            TRANSACTION_COLUMN_FAMILY_NAME,
            TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
            TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME,
            STATE_CHANGE_SET_COLUMN_FAMILY_NAME,
        ];

        // save sequencer info and startup info for setup with previous tx values
        if update_startup {
            let previous_accumulator_info = previous_accumulator_info_opt.ok_or_else(|| {
                anyhow::anyhow!("revert tx failed: previous_accumulator_info not found")
            })?;
            let previous_execution_info = previous_execution_info_opt.ok_or_else(|| {
                anyhow::anyhow!("revert tx failed: previous_execution_info not found")
            })?;
            let previous_sequencer_info =
                SequencerInfo::new(tx_order - 1, previous_accumulator_info);
            let startup_info = moveos_types::startup_info::StartupInfo::new(
                previous_execution_info.state_root,
                previous_execution_info.size,
            );
            write_batch.put(
                to_bytes(SEQUENCER_INFO_KEY)?,
                to_bytes(&previous_sequencer_info)?,
            )?;
            write_batch.put(to_bytes(STARTUP_INFO_KEY)?, to_bytes(&startup_info)?)?;
            cf_names.push(META_SEQUENCER_INFO_COLUMN_FAMILY_NAME);
            cf_names.push(CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME);
        }

        inner_store.write_batch_across_cfs(cf_names, write_batch, true)?;

        // revert the indexer
        self.revert_indexer(tx_order, state_change_set_ext_opt)
    }

    fn revert_indexer(
        &self,
        tx_order: u64,
        state_change_set_ext_opt: Option<StateChangeSetExt>,
    ) -> Result<()> {
        let previous_state_change_set_ext_opt =
            self.rooch_store.get_state_change_set(tx_order - 1)?;
        if previous_state_change_set_ext_opt.is_some() && state_change_set_ext_opt.is_some() {
            let previous_state_change_set_ext = previous_state_change_set_ext_opt.unwrap();
            let state_change_set_ext = state_change_set_ext_opt.unwrap();

            let mut object_ids = vec![];
            for (_feild_key, object_change) in state_change_set_ext.state_change_set.changes.clone()
            {
                collect_revert_object_change_ids(object_change, &mut object_ids)?;
            }

            let root = ObjectMeta::root_metadata(
                previous_state_change_set_ext.state_change_set.state_root,
                previous_state_change_set_ext.state_change_set.global_size,
            );
            let resolver = RootObjectResolver::new(root, &self.moveos_store);
            let object_mapping = resolver
                .get_states(AccessPath::objects(object_ids))?
                .into_iter()
                .flatten()
                .map(|v| (v.metadata.id.clone(), v.metadata))
                .collect::<HashMap<_, _>>();

            // 1. revert indexer transaction
            self.indexer_store
                .delete_transactions(vec![tx_order])
                .map_err(|e| anyhow!(format!("Revert indexer transactions error: {:?}", e)))?;

            // 2. revert indexer event
            self.indexer_store
                .delete_events(vec![tx_order])
                .map_err(|e| anyhow!(format!("Revert indexer events error: {:?}", e)))?;

            // 3. revert indexer full object state, including object_states, utxos and inscriptions
            // indexer object state index generator
            let mut state_index_generator = IndexerObjectStatesIndexGenerator::default();
            let mut indexer_object_state_change_set = IndexerObjectStateChangeSet::default();

            for (_feild_key, object_change) in state_change_set_ext.state_change_set.changes {
                handle_revert_object_change(
                    &mut state_index_generator,
                    tx_order,
                    &mut indexer_object_state_change_set,
                    object_change,
                    &object_mapping,
                )?;
            }
            self.indexer_store
                .apply_object_states(indexer_object_state_change_set)
                .map_err(|e| anyhow!(format!("Revert indexer states error: {:?}", e)))?;
        };
        Ok(())
    }

    // check the moveos store:
    // last execution info match state root
    fn check_moveos_store_thorough(&self) -> anyhow::Result<()> {
        let mut last_order = self
            .rooch_store
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("Sequencer info not found"))?
            .last_order;
        if last_order == 0 {
            return Ok(()); // Only genesis
        }
        let mut state_root = H256::default();
        for order in (1..=last_order).rev() {
            let tx_hash = self.rooch_store.get_tx_hashes(vec![order])?.pop().flatten();
            if let Some(tx_hash) = tx_hash {
                let execution_info = self.moveos_store.get_tx_execution_info(tx_hash)?;
                if let Some(execution_info) = execution_info {
                    state_root = execution_info.state_root;
                    last_order = order;
                    break; // found the last execution info
                }
            }
        }
        let startup_info = self.moveos_store.config_store.get_startup_info()?;
        let startup_state_root = startup_info
            .map(|s| s.state_root)
            .ok_or_else(|| anyhow::anyhow!("Startup info not found"))?;

        if state_root != startup_state_root {
            return Err(anyhow!(
                "State root mismatch: last execution info state root {:?} for order: {}, startup state root {:?}",
                state_root,
                last_order,
                startup_state_root
            ));
        }

        Ok(())
    }

    /// repair the rooch store, return the (issues count, fixed count)
    /// if exec is false, only report issues, otherwise repair the issues
    pub fn repair(&self, thorough: bool, exec: bool) -> anyhow::Result<(usize, usize)> {
        let mut issues = 0;
        let mut fixed = 0;
        // repair the rooch store
        let (rooch_store_issues, rooch_store_fixed) = self.rooch_store.repair(thorough, exec)?;
        issues += rooch_store_issues;
        fixed += rooch_store_fixed;
        // check moveos store
        if thorough {
            match self.check_moveos_store_thorough() {
                Ok(_) => {}
                Err(e) => {
                    issues += 1;
                    error!("MoveOS store check failed: {:?}", e);
                }
            }
        }
        // TODO repair the changeset sync and indexer store
        Ok((issues, fixed))
    }

    //TODO implements this after proposer generate blocks
    pub fn l2_block_ref_by_number(&self, block_number: u64) -> Result<L2BlockRef> {
        return Ok(L2BlockRef::default());
    }
}
