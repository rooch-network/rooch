// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Error, Result};
use moveos_store::config_store::ConfigStore;
use moveos_store::transaction_store::TransactionStore as TxExecutionInfoStore;
use moveos_store::MoveOSStore;
use moveos_types::access_path::AccessPath;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state_resolver::{RootObjectResolver, StateReader};
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
use raw_store::{rocks::RocksDB, StoreInstance};
use rooch_config::store_config::StoreConfig;
use rooch_indexer::store::traits::IndexerStoreTrait;
use rooch_indexer::{indexer_reader::IndexerReader, IndexerStore};
use rooch_store::state_store::StateStore;
use rooch_store::RoochStore;
use rooch_types::indexer::state::{
    collect_revert_object_change_ids, handle_revert_object_change, IndexerObjectStateChangeSet,
    IndexerObjectStatesIndexGenerator,
};
use rooch_types::sequencer::SequencerInfo;

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
        let rooch_store = RoochStore::new_with_instance(instance, registry)?;
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
        let registry = prometheus::Registry::new();
        Self::init(config, &registry)
    }

    pub fn latest_root(&self) -> Result<Option<ObjectMeta>> {
        let startup_info = self.moveos_store.config_store.get_startup_info()?;

        Ok(startup_info.map(|s| s.into_root_metadata()))
    }

    pub fn revert_tx(&self, tx_hash: H256) -> Result<()> {
        let last_sequencer_info = self
            .rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("Load sequencer info failed"))?;
        let (last_order, last_accumulator_info) = (
            last_sequencer_info.last_order,
            last_sequencer_info.last_accumulator_info.clone(),
        );
        println!("Load latest sequencer order {:?}", last_order);
        println!(
            "Load latest sequencer accumulator info {:?}",
            last_accumulator_info
        );

        let ledger_tx_opt = self
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)?;
        if ledger_tx_opt.is_none() {
            println!("the ledger tx not exist via tx_hash {}", tx_hash);
            return Ok(());
        }
        let sequencer_info = ledger_tx_opt.unwrap().sequence_info;
        let tx_order = sequencer_info.tx_order;
        assert_eq!(
            sequencer_info.tx_order, last_sequencer_info.last_order,
            "the last order {} should match current sequencer info tx order {}, tx_hash {}",
            last_sequencer_info.last_order, sequencer_info.tx_order, tx_hash
        );

        self.do_revert_tx_ignore_check(tx_hash)?;

        let previous_tx_order = if tx_order > 0 { tx_order - 1 } else { 0 };
        let previous_tx_hash_opt = self
            .rooch_store
            .transaction_store
            .get_tx_hashes(vec![previous_tx_order])?;
        if previous_tx_hash_opt.is_empty() || previous_tx_hash_opt[0].is_none() {
            return Err(Error::msg(format!(
                "the previous tx hash not exist via previous_tx_order  {}",
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
                "the previous ledger tx not exist via tx_hash {}, revert tx failed",
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
        let previous_sequencer_info = previous_ledger_tx_opt.unwrap().sequence_info;
        let previous_execution_info = previous_execution_info_opt.unwrap();
        let revert_sequencer_info = SequencerInfo::new(
            previous_tx_order,
            previous_sequencer_info.tx_accumulator_info(),
        );
        self.rooch_store
            .meta_store
            .save_sequencer_info_ignore_check(revert_sequencer_info)?;

        let startup_info = StartupInfo::new(
            previous_execution_info.state_root,
            previous_execution_info.size,
        );
        self.moveos_store.save_startup_info(startup_info)?;

        println!(
            "revert tx succ, tx_hash: {:?}, tx_order {}",
            tx_hash, tx_order
        );

        Ok(())
    }

    pub fn do_revert_tx_ignore_check(&self, tx_hash: H256) -> Result<()> {
        let ledger_tx_opt = self
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)?;

        if ledger_tx_opt.is_none() {
            println!("the ledger tx not exist via tx_hash {}", tx_hash);
            return Ok(());
        }

        let sequencer_info = ledger_tx_opt.unwrap().sequence_info;
        let tx_order = sequencer_info.tx_order;
        self.rooch_store
            .transaction_store
            .remove_transaction(tx_hash, tx_order)?;
        self.moveos_store
            .transaction_store
            .remove_tx_execution_info(tx_hash)?;

        // remove the state change set
        let state_change_set_ext_opt = self.rooch_store.get_state_change_set(tx_order)?;
        if state_change_set_ext_opt.is_some() {
            self.rooch_store.remove_state_change_set(tx_order)?;
        }

        // revert the indexer
        let previous_tx_order = if tx_order > 0 { tx_order - 1 } else { 0 };
        let previous_state_change_set_ext_opt =
            self.rooch_store.get_state_change_set(previous_tx_order)?;
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
        }
        println!(
            "revert tx and indexer succ, tx_hash: {:?}, tx_order {}",
            tx_hash, tx_order
        );

        Ok(())
    }
}
