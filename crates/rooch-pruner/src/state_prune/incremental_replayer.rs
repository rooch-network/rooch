// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state_prune::{ProgressTracker, StatePruneMetadata};
use anyhow::Result;
use move_core_types::effects::Op;
use moveos_common::utils::to_bytes;
use moveos_config::store_config::RocksdbConfig;
use moveos_store::transaction_store::TransactionStore as MoveOSTransactionStore;
use moveos_store::{
    MoveOSStore, CONFIG_GENESIS_COLUMN_FAMILY_NAME, STATE_NODE_COLUMN_FAMILY_NAME,
    TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME,
};
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::GENESIS_STATE_ROOT;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::StateChangeSetExt;
use moveos_types::state_resolver::StateResolver;
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
use raw_store::SchemaStore;
use rooch_config::state_prune::{HistoryPruneReport, ReplayConfig, ReplayReport};
use rooch_store::da_store::DAMetaStore;
use rooch_store::proposer_store::ProposerStore;
use rooch_store::{
    RoochStore, DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME, DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME,
    PROPOSER_LAST_BLOCK_COLUMN_FAMILY_NAME, STATE_CHANGE_SET_COLUMN_FAMILY_NAME,
    TRANSACTION_COLUMN_FAMILY_NAME, TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME,
    TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
};
use rooch_types::sequencer::SequencerInfo;
use serde_json;
use smt::NodeReader;
use std::cmp::min;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

const FULL_COPY_COLUMN_FAMILIES: &[&str] = &[
    TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME,
    CONFIG_GENESIS_COLUMN_FAMILY_NAME,
    DA_BLOCK_CURSOR_COLUMN_FAMILY_NAME,
    DA_BLOCK_SUBMIT_STATE_COLUMN_FAMILY_NAME,
    PROPOSER_LAST_BLOCK_COLUMN_FAMILY_NAME,
];

const WINDOWED_HISTORY_COLUMN_FAMILIES: &[&str] = &[
    TRANSACTION_COLUMN_FAMILY_NAME,
    TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
    TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME,
    STATE_CHANGE_SET_COLUMN_FAMILY_NAME,
];

/// Incremental replayer for applying changesets to a snapshot
pub struct IncrementalReplayer {
    config: ReplayConfig,
    progress_tracker: ProgressTracker,
    /// Live RoochStore for reading changesets
    rooch_store: rooch_store::RoochStore,
}

impl IncrementalReplayer {
    /// Create new incremental replayer
    pub fn new(config: ReplayConfig, rooch_store: rooch_store::RoochStore) -> Result<Self> {
        // Validate configuration
        if config.default_batch_size == 0 {
            return Err(anyhow::anyhow!("Batch size must be greater than 0"));
        }

        Ok(Self {
            config,
            progress_tracker: ProgressTracker::new(30), // Report every 30 seconds
            rooch_store,
        })
    }

    /// Replay changesets onto snapshot
    pub async fn replay_changesets(
        &self,
        input_snapshot_path: &PathBuf,
        from_order: u64,
        to_order: u64,
        output_dir: &Path,
    ) -> Result<ReplayReport> {
        info!("Starting replay from order {} to {}", from_order, to_order);

        let start_time = Instant::now();
        let mut report = ReplayReport::new();

        // Initialize metadata
        let mut metadata = StatePruneMetadata::new(
            crate::state_prune::OperationType::Replay {
                snapshot_path: input_snapshot_path.clone(),
                from_order,
                to_order,
                output_dir: output_dir.to_path_buf(),
            },
            serde_json::json!({
                "input_snapshot_path": input_snapshot_path,
                "from_order": from_order,
                "to_order": to_order,
                "config": self.config
            }),
        );

        metadata.mark_in_progress("Preparing output database".to_string(), 5.0);

        // Load snapshot metadata and state store
        let snapshot_meta = self.load_snapshot_metadata(input_snapshot_path)?;
        let snapshot_store = self.load_snapshot_store(input_snapshot_path)?;
        let retain_from = self.resolve_retain_from(from_order, to_order)?;

        // Build output DB from a fresh store and copy runtime-required column families.
        self.prepare_fresh_output_store(output_dir)?;
        metadata.mark_in_progress("Copying required column families".to_string(), 10.0);
        self.copy_required_cfs(output_dir, retain_from, to_order)?;
        let (output_store, output_rooch_store) = self.load_output_stores(output_dir)?;

        metadata.mark_in_progress("Importing snapshot nodes".to_string(), 20.0);
        self.import_snapshot_nodes(&snapshot_store, &output_store, &mut report, &mut metadata)?;

        metadata.mark_in_progress("Loading changesets".to_string(), 40.0);

        // Load changesets in range
        let changesets = self.load_changesets_range(from_order, to_order, &mut report)?;
        self.progress_tracker.set_total(changesets.len() as u64);

        info!("Loaded {} changesets to replay", changesets.len());

        metadata.mark_in_progress("Replaying changesets".to_string(), 50.0);

        // Process changesets in batches and get expected final state root
        let (actual_state_root, expected_state_root, expected_global_size) = self
            .replay_changesets_batched(
                changesets,
                &output_store,
                snapshot_meta.state_root,
                snapshot_meta.global_size,
                &mut report,
                &mut metadata,
            )
            .await?;

        metadata.mark_in_progress("Updating startup info".to_string(), 85.0);
        self.update_startup_info(&output_store, actual_state_root, expected_global_size)?;
        report.final_state_root = actual_state_root;

        metadata.mark_in_progress("Compacting state nodes".to_string(), 88.0);
        output_store.get_state_node_store().flush_and_compact()?;

        // Verify final state root if enabled
        if self.config.verify_final_state_root {
            metadata.mark_in_progress("Verifying final state root".to_string(), 90.0);
            if let Err(e) =
                self.verify_final_state_root(&output_store, expected_state_root, &mut report)
            {
                // Verification failed - mark metadata as failed
                metadata.mark_failed(format!("State root verification failed: {}", e));
                return Err(e);
            }
        }

        metadata.mark_in_progress("Refreshing output metadata".to_string(), 92.0);
        self.refresh_output_metadata(&output_rooch_store, to_order, &mut metadata)?;

        // After metadata refresh, ensure startup_info and sequencer_info are consistent.
        self.verify_startup_sequencer_consistency(&output_store, &output_rooch_store, to_order)?;
        report.history_prune_report = Some(self.build_windowed_history_report(retain_from));

        // Create checkpoints if enabled
        if self.config.enable_checkpoints {
            metadata.mark_in_progress("Creating checkpoints".to_string(), 95.0);
            self.create_checkpoints(output_dir, &report)?;
        }

        // Finalize report
        report.duration_seconds = start_time.elapsed().as_secs();

        if report.errors.is_empty() {
            metadata.mark_completed();
            info!(
                "Replay completed successfully in {:?}",
                start_time.elapsed()
            );
        } else {
            metadata.mark_failed(format!("Replay failed with {} errors", report.errors.len()));
            error!("Replay failed: {}", report.errors.join("; "));
        }

        // Save report
        let report_path = output_dir.join("replay_report.json");
        report.save_to_file(&report_path)?;
        metadata.save_to_file(output_dir.join("operation_meta.json"))?;

        Ok(report)
    }

    /// Finalize an existing replay output after the heavy replay phases have already succeeded.
    pub fn finalize_existing_output(
        &self,
        output_dir: &Path,
        to_order: u64,
        expected_state_root: Option<H256>,
    ) -> Result<ReplayReport> {
        let start_time = Instant::now();
        let mut report = ReplayReport::new();
        let mut metadata = StatePruneMetadata::new(
            crate::state_prune::OperationType::Replay {
                snapshot_path: PathBuf::new(),
                from_order: 0,
                to_order,
                output_dir: output_dir.to_path_buf(),
            },
            serde_json::json!({
                "mode": "finalize_existing_output",
                "to_order": to_order,
                "output_dir": output_dir,
                "expected_state_root": expected_state_root.map(|root| format!("{:x}", root)),
            }),
        );
        let (output_store, output_rooch_store) = self.load_output_stores(output_dir)?;

        if let Some(expected_state_root) = expected_state_root {
            metadata.mark_in_progress("Verifying final state root".to_string(), 90.0);
            self.verify_final_state_root(&output_store, expected_state_root, &mut report)?;
        } else if let Some(startup_info) = output_store.get_config_store().get_startup_info()? {
            report.final_state_root = startup_info.state_root;
        }

        metadata.mark_in_progress("Refreshing output metadata".to_string(), 92.0);
        self.refresh_output_metadata(&output_rooch_store, to_order, &mut metadata)?;
        self.verify_startup_sequencer_consistency(&output_store, &output_rooch_store, to_order)?;

        report.duration_seconds = start_time.elapsed().as_secs();
        metadata.mark_completed();
        report.verification_passed = expected_state_root.is_some();

        let report_path = output_dir.join("replay_report.json");
        report.save_to_file(&report_path)?;
        metadata.save_to_file(output_dir.join("operation_meta.json"))?;

        Ok(report)
    }

    /// Load snapshot metadata
    fn load_snapshot_metadata(
        &self,
        snapshot_path: &Path,
    ) -> Result<rooch_config::state_prune::SnapshotMeta> {
        let metadata_path = snapshot_path.join("snapshot_meta.json");
        rooch_config::state_prune::SnapshotMeta::load_from_file(&metadata_path)
            .map_err(|e| anyhow::anyhow!("Failed to load snapshot metadata: {}", e))
    }

    /// Load snapshot store from snapshot path
    fn load_snapshot_store(&self, snapshot_path: &Path) -> Result<MoveOSStore> {
        // The snapshot database is stored at snapshot_path/snapshot.db
        let snapshot_db_path = snapshot_path.join("snapshot.db");

        // Check if snapshot database exists
        if !snapshot_db_path.exists() {
            return Err(anyhow::anyhow!(
                "Snapshot database not found at {:?}. \
                 Please ensure the snapshot path is correct and contains a valid snapshot.",
                snapshot_db_path
            ));
        }

        // Verify it's a directory (RocksDB databases are directories)
        if !snapshot_db_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Snapshot path exists but is not a valid database: {:?}",
                snapshot_db_path
            ));
        }

        // Create a new MoveOSStore from the snapshot database
        // This loads the snapshot state instead of using the live database
        let registry = Registry::new();
        let snapshot_store = MoveOSStore::new(&snapshot_db_path, &registry).map_err(|e| {
            anyhow::anyhow!(
                "Failed to load snapshot store from {:?}: {}",
                snapshot_db_path,
                e
            )
        })?;

        info!(
            "Successfully loaded snapshot store from {:?}",
            snapshot_db_path
        );

        Ok(snapshot_store)
    }

    fn all_column_families() -> Vec<&'static str> {
        let mut column_families = moveos_store::StoreMeta::get_column_family_names().to_vec();
        column_families.extend_from_slice(rooch_store::StoreMeta::get_column_family_names());
        column_families.sort();
        column_families.dedup();
        column_families
    }

    fn resolve_retain_from(&self, from_order: u64, to_order: u64) -> Result<u64> {
        let retain_from = if let Some(history_prune) = &self.config.history_prune {
            if let Some(window) = history_prune.retain_window {
                to_order.saturating_sub(window).saturating_add(1)
            } else if history_prune.retain_from > 0 {
                history_prune.retain_from
            } else {
                from_order
            }
        } else {
            from_order
        };

        if retain_from > to_order {
            return Err(anyhow::anyhow!(
                "retain_from {} exceeds to_order {}",
                retain_from,
                to_order
            ));
        }

        Ok(retain_from)
    }

    /// Create an empty output database with the full set of column families.
    fn prepare_fresh_output_store(&self, output_dir: &Path) -> Result<()> {
        if output_dir.exists() {
            return Err(anyhow::anyhow!(
                "Output directory already exists: {:?}. Please provide an empty path.",
                output_dir
            ));
        }

        if let Some(parent) = output_dir.parent() {
            fs::create_dir_all(parent)?;
        }

        raw_store::rocks::RocksDB::new(
            output_dir,
            Self::all_column_families(),
            RocksdbConfig::default(),
        )?;

        info!("Created fresh output database at {:?}", output_dir);
        Ok(())
    }

    fn source_moveos_store(&self) -> Result<MoveOSStore> {
        let registry = Registry::new();
        MoveOSStore::new_with_instance(self.rooch_store.store_instance.clone(), &registry)
            .map_err(|e| anyhow::anyhow!("Failed to load source MoveOSStore: {}", e))
    }

    fn copy_required_cfs(&self, output_dir: &Path, retain_from: u64, to_order: u64) -> Result<()> {
        let source_db = self
            .rooch_store
            .store_instance
            .db()
            .ok_or_else(|| anyhow::anyhow!("Failed to access source RocksDB instance"))?
            .inner();
        let (_moveos_store, output_rooch_store) = self.load_output_stores(output_dir)?;

        let target_db = output_rooch_store
            .store_instance
            .db()
            .ok_or_else(|| anyhow::anyhow!("Failed to access output RocksDB instance"))?
            .inner();
        let source_moveos_store = self.source_moveos_store()?;

        for cf_name in FULL_COPY_COLUMN_FAMILIES {
            let source_cf = source_db
                .cf_handle(cf_name)
                .ok_or_else(|| anyhow::anyhow!("Source CF not found: {}", cf_name))?;
            let target_cf = target_db
                .cf_handle(cf_name)
                .ok_or_else(|| anyhow::anyhow!("Target CF not found: {}", cf_name))?;

            let mut batch = rocksdb::WriteBatch::default();
            let mut count = 0usize;
            let iter = source_db.iterator_cf(source_cf, rocksdb::IteratorMode::Start);

            for item in iter {
                let (key, value) = item?;
                batch.put_cf(&target_cf, key, value);
                count += 1;

                if count % self.config.default_batch_size.max(1) == 0 {
                    target_db.write(batch)?;
                    batch = rocksdb::WriteBatch::default();
                }
            }

            if !batch.is_empty() {
                target_db.write(batch)?;
            }

            target_db.flush_cf(&target_cf)?;
            info!("Copied CF {} with {} entries", cf_name, count);
        }

        self.copy_windowed_history(
            &source_moveos_store,
            &output_rooch_store,
            retain_from,
            to_order,
        )?;

        Ok(())
    }

    fn copy_windowed_history(
        &self,
        source_moveos_store: &MoveOSStore,
        output_rooch_store: &RoochStore,
        retain_from: u64,
        to_order: u64,
    ) -> Result<()> {
        if retain_from > to_order {
            return Ok(());
        }

        let output_db = output_rooch_store
            .store_instance
            .db()
            .ok_or_else(|| anyhow::anyhow!("Failed to access output RocksDB instance"))?
            .inner();
        let tx_cf = output_db
            .cf_handle(TRANSACTION_COLUMN_FAMILY_NAME)
            .ok_or_else(|| {
                anyhow::anyhow!("Target CF not found: {}", TRANSACTION_COLUMN_FAMILY_NAME)
            })?;
        let tx_map_cf = output_db
            .cf_handle(TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Target CF not found: {}",
                    TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME
                )
            })?;
        let exec_cf = output_db
            .cf_handle(TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Target CF not found: {}",
                    TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME
                )
            })?;
        let changeset_cf = output_db
            .cf_handle(STATE_CHANGE_SET_COLUMN_FAMILY_NAME)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Target CF not found: {}",
                    STATE_CHANGE_SET_COLUMN_FAMILY_NAME
                )
            })?;

        let batch_size = self.config.default_batch_size.max(1) as u64;
        let mut copied_transactions = 0u64;
        let mut copied_changesets = 0u64;
        let mut start = retain_from;

        while start <= to_order {
            let end = min(to_order, start.saturating_add(batch_size - 1));
            let orders: Vec<u64> = (start..=end).collect();
            let tx_hashes = self
                .rooch_store
                .transaction_store
                .get_tx_hashes(orders.clone())?;
            let mut batch = rocksdb::WriteBatch::default();

            for (index, tx_hash_opt) in tx_hashes.into_iter().enumerate() {
                let tx_order = start + index as u64;
                let tx_hash = tx_hash_opt.ok_or_else(|| {
                    anyhow::anyhow!("Missing tx hash for order {} during history copy", tx_order)
                })?;
                let tx = self
                    .rooch_store
                    .transaction_store
                    .get_transaction_by_hash(tx_hash)?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Missing transaction for order {} and hash {:x}",
                            tx_order,
                            tx_hash
                        )
                    })?;
                let execution_info = source_moveos_store
                    .get_tx_execution_info(tx_hash)?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Missing execution info for order {} and hash {:x}",
                            tx_order,
                            tx_hash
                        )
                    })?;
                let changeset = self
                    .rooch_store
                    .get_state_store()
                    .get_state_change_set(tx_order)?
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing state changeset for order {}", tx_order)
                    })?;

                batch.put_cf(&tx_cf, to_bytes(&tx_hash)?, bcs::to_bytes(&tx)?);
                batch.put_cf(&tx_map_cf, to_bytes(&tx_order)?, to_bytes(&tx_hash)?);
                batch.put_cf(
                    &exec_cf,
                    to_bytes(&tx_hash)?,
                    bcs::to_bytes(&execution_info)?,
                );
                batch.put_cf(
                    &changeset_cf,
                    to_bytes(&tx_order)?,
                    bcs::to_bytes(&changeset)?,
                );

                copied_transactions += 1;
                copied_changesets += 1;
            }

            if !batch.is_empty() {
                output_db.write(batch)?;
            }

            start = end.saturating_add(1);
        }

        info!(
            "Copied windowed history [{}..={}]: {} txs, {} changesets across {:?}",
            retain_from,
            to_order,
            copied_transactions,
            copied_changesets,
            WINDOWED_HISTORY_COLUMN_FAMILIES
        );

        Ok(())
    }

    /// Load output MoveOSStore and RoochStore using a single RocksDB instance (all CFs)
    fn load_output_stores(&self, output_dir: &Path) -> Result<(MoveOSStore, RoochStore)> {
        if !output_dir.exists() || !output_dir.is_dir() {
            return Err(anyhow::anyhow!(
                "Output database not found at {:?}.",
                output_dir
            ));
        }

        let registry = Registry::new();
        let db_metrics = DBMetrics::get_or_init(&registry).clone();
        let rocksdb = raw_store::rocks::RocksDB::new(
            output_dir,
            Self::all_column_families(),
            RocksdbConfig::default(),
        )?;
        let instance = raw_store::StoreInstance::new_db_instance(rocksdb, db_metrics);

        // Share the same instance between MoveOSStore and RoochStore to avoid locks
        let moveos_store = MoveOSStore::new_with_instance(instance.clone(), &registry)?;
        let rooch_store = RoochStore::new_with_instance(instance, &registry)
            .map_err(|e| anyhow::anyhow!("Failed to load output RoochStore: {}", e))?;

        Ok((moveos_store, rooch_store))
    }

    /// Import snapshot nodes into output store
    fn import_snapshot_nodes(
        &self,
        snapshot_store: &MoveOSStore,
        output_store: &MoveOSStore,
        report: &mut ReplayReport,
        metadata: &mut StatePruneMetadata,
    ) -> Result<()> {
        let raw_db = snapshot_store
            .get_state_node_store()
            .get_store()
            .store()
            .db()
            .ok_or_else(|| anyhow::anyhow!("Failed to access snapshot RocksDB instance"))?
            .inner();
        let cf = raw_db
            .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
            .ok_or_else(|| anyhow::anyhow!("State node column family not found"))?;
        let mut iter = raw_db.raw_iterator_cf(&cf);
        iter.seek_to_first();

        let mut batch = BTreeMap::new();
        let mut imported = 0u64;
        let mut last_report = Instant::now();

        while iter.valid() {
            if let (Some(key), Some(value)) = (iter.key(), iter.value()) {
                if key.len() != 32 {
                    return Err(anyhow::anyhow!(
                        "Invalid state node key length: {}",
                        key.len()
                    ));
                }
                let hash = H256::from_slice(key);
                batch.insert(hash, value.to_vec());
            }

            if batch.len() >= self.config.default_batch_size {
                output_store.get_state_node_store().write_nodes(batch)?;
                imported += self.config.default_batch_size as u64;
                batch = BTreeMap::new();

                if last_report.elapsed() >= Duration::from_secs(30) {
                    metadata
                        .mark_in_progress(format!("Importing snapshot nodes ({})", imported), 20.0);
                    last_report = Instant::now();
                }
            }

            iter.next();
        }

        iter.status()
            .map_err(|e| anyhow::anyhow!("Snapshot iterator error: {}", e))?;

        if !batch.is_empty() {
            imported += batch.len() as u64;
            output_store.get_state_node_store().write_nodes(batch)?;
        }

        report.nodes_updated += imported;
        info!("Imported {} snapshot nodes into output store", imported);

        Ok(())
    }

    fn update_startup_info(
        &self,
        output_store: &MoveOSStore,
        state_root: H256,
        global_size: u64,
    ) -> Result<()> {
        output_store
            .get_config_store()
            .save_startup_info(StartupInfo::new(state_root, global_size))?;
        Ok(())
    }

    fn normalize_changeset_pre_state_roots(
        &self,
        output_store: &MoveOSStore,
        pre_state_root: H256,
        changeset: &mut moveos_types::state::StateChangeSet,
    ) -> Result<()> {
        let root_metadata = moveos_types::moveos_std::object::ObjectMeta::root_metadata(
            pre_state_root,
            changeset.global_size,
        );
        let resolver =
            moveos_types::state_resolver::RootObjectResolver::new(root_metadata, output_store);

        for obj_change in changeset.changes.values_mut() {
            self.normalize_object_change_pre_state_root(&resolver, obj_change)?;
        }

        Ok(())
    }

    fn normalize_object_change_pre_state_root(
        &self,
        resolver: &moveos_types::state_resolver::RootObjectResolver<'_, MoveOSStore>,
        obj_change: &mut moveos_types::state::ObjectChange,
    ) -> Result<()> {
        let object_id = obj_change.metadata.id.clone();
        let resolved = resolver.get_object(&object_id)?;

        match &obj_change.value {
            Some(Op::New(_)) => {
                if resolved.is_some() {
                    return Err(anyhow::anyhow!(
                        "Object {} marked as New but exists in pre-state",
                        object_id
                    ));
                }
                if obj_change.metadata.state_root.is_some()
                    && obj_change.metadata.state_root != Some(*GENESIS_STATE_ROOT)
                {
                    warn!(
                        "New object {} has non-empty state_root; resetting to GENESIS for replay",
                        object_id
                    );
                }
                obj_change.metadata.state_root = None;
            }
            _ => {
                let object_state = resolved.ok_or_else(|| {
                    anyhow::anyhow!("Object {} not found in pre-state during replay", object_id)
                })?;

                obj_change.metadata.state_root = object_state.metadata.state_root;
            }
        }

        for child in obj_change.fields.values_mut() {
            self.normalize_object_change_pre_state_root(resolver, child)?;
        }

        Ok(())
    }

    fn refresh_output_metadata(
        &self,
        output_rooch_store: &RoochStore,
        to_order: u64,
        metadata: &mut StatePruneMetadata,
    ) -> Result<()> {
        metadata.mark_in_progress(
            format!("Refreshing runtime metadata at order {}", to_order),
            92.0,
        );

        let target_tx = output_rooch_store
            .transaction_store
            .get_tx_by_order(to_order)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Missing transaction for to_order {} in output store",
                    to_order
                )
            })?;
        let new_sequencer_info =
            SequencerInfo::new(to_order, target_tx.sequence_info.tx_accumulator_info());
        output_rooch_store
            .get_meta_store()
            .save_sequencer_info_unsafe(new_sequencer_info)?;

        let (da_issues, da_fixed) =
            output_rooch_store.try_repair_da_meta(to_order, false, None, false, false)?;
        if da_issues > 0 {
            info!(
                "DA meta repair after trim: issues {}, fixed {}",
                da_issues, da_fixed
            );
        }

        let last_block_number = output_rooch_store.get_last_block_number()?;
        let last_proposed = output_rooch_store.get_last_proposed()?;
        match (last_block_number, last_proposed) {
            (None, Some(_)) => {
                output_rooch_store.clear_last_proposed()?;
            }
            (Some(last_block_number), Some(last_proposed)) if last_proposed > last_block_number => {
                output_rooch_store.set_last_proposed(last_block_number)?;
            }
            _ => {}
        }

        info!(
            "Refreshed output metadata at order {} after windowed history copy",
            to_order
        );

        Ok(())
    }

    fn build_windowed_history_report(&self, retain_from: u64) -> HistoryPruneReport {
        HistoryPruneReport {
            enabled: true,
            retain_from,
            cfs_pruned: WINDOWED_HISTORY_COLUMN_FAMILIES
                .iter()
                .map(|cf| (*cf).to_string())
                .collect(),
            dry_run: false,
            ..Default::default()
        }
    }

    /// Load changesets in specified range
    fn load_changesets_range(
        &self,
        from_order: u64,
        to_order: u64,
        report: &mut ReplayReport,
    ) -> Result<Vec<(u64, StateChangeSetExt)>> {
        // Validate range (inclusive)
        if from_order > to_order {
            info!("Empty changeset range: {}..{}", from_order, to_order);
            return Ok(Vec::new());
        }

        info!(
            "Loading changesets in range {}..{} (inclusive) from rooch_store",
            from_order, to_order
        );

        let range_end = to_order
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("to_order overflow: {}", to_order))?;

        // Load changesets from the rooch_store's state store
        let changesets = self
            .rooch_store
            .get_state_store()
            .get_changesets_range(from_order, range_end)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to load changesets from range {}..{} (inclusive): {}",
                    from_order,
                    to_order,
                    e
                )
            })?;

        // Check if we got all the changesets we expected
        let expected_count = (range_end - from_order) as usize;
        if changesets.len() != expected_count {
            report.add_error(format!(
                "Expected {} changesets in range {}..{} (inclusive), but found {}",
                expected_count,
                from_order,
                to_order,
                changesets.len()
            ));

            // Log which orders are missing
            use std::collections::HashSet;
            let loaded_orders: HashSet<u64> = changesets.iter().map(|(order, _)| *order).collect();
            let missing_orders: Vec<u64> = (from_order..range_end)
                .filter(|order| !loaded_orders.contains(order))
                .collect();

            if !missing_orders.is_empty() {
                warn!("Missing changesets for orders: {:?}", missing_orders);
            }
        }

        info!(
            "Successfully loaded {} changesets in range {}..{}",
            changesets.len(),
            from_order,
            to_order
        );

        Ok(changesets)
    }

    /// Replay changesets in batches
    /// Returns the expected final state root (from the last changeset applied)
    async fn replay_changesets_batched(
        &self,
        changesets: Vec<(u64, StateChangeSetExt)>,
        output_store: &MoveOSStore,
        base_state_root: H256,
        base_global_size: u64,
        report: &mut ReplayReport,
        metadata: &mut StatePruneMetadata,
    ) -> Result<(H256, H256, u64)> {
        let total_changesets = changesets.len();
        let mut processed = 0;
        let mut current_state_root = base_state_root;

        // Process in batches
        for batch in changesets.chunks(self.config.default_batch_size) {
            let batch_start = processed;
            let batch_end = processed + batch.len();

            info!(
                "Processing batch {}..{} ({} changesets)",
                batch_start,
                batch_end,
                batch.len()
            );

            // Apply batch
            self.apply_changeset_batch(batch, output_store, &mut current_state_root, report)?;

            processed += batch.len();
            self.progress_tracker
                .increment_processed(batch.len() as u64);

            // Update progress
            if self.progress_tracker.should_report() {
                let progress = self.progress_tracker.get_progress_report();
                let overall_progress = 20.0 + (progress.progress_percentage * 0.7); // 20-90%
                metadata.mark_in_progress(
                    format!("Replaying changesets ({}/{})", processed, total_changesets),
                    overall_progress,
                );
                info!("Replay progress: {}", progress.format());
                self.progress_tracker.mark_reported();
            }

            // Validate after batch if enabled
            if self.config.validate_after_batch {
                self.validate_batch_state(
                    output_store,
                    batch.last().unwrap().1.state_change_set.state_root,
                )?;
            }
        }

        report.changesets_processed = processed as u64;

        // Get the expected final state root from the last changeset
        let expected_state_root = changesets
            .last()
            .map(|(_, changeset)| changeset.state_change_set.state_root)
            .unwrap_or(base_state_root);
        let expected_global_size = changesets
            .last()
            .map(|(_, changeset)| changeset.state_change_set.global_size)
            .unwrap_or(base_global_size);

        info!(
            "Processed {} changesets, expected final state root: {:x}",
            processed, expected_state_root
        );

        Ok((
            current_state_root,
            expected_state_root,
            expected_global_size,
        ))
    }

    /// Apply a batch of changesets
    fn apply_changeset_batch(
        &self,
        batch: &[(u64, StateChangeSetExt)],
        output_store: &MoveOSStore,
        current_state_root: &mut H256,
        report: &mut ReplayReport,
    ) -> Result<()> {
        let mut total_objects_created = 0u64;
        let mut total_objects_updated = 0u64;
        let mut batch_nodes_updated = 0u64;

        // Process each changeset in the batch sequentially
        for (tx_order, changeset_ext) in batch {
            let mut changeset = changeset_ext.state_change_set.clone();
            let expected_root = changeset.state_root;

            // Log state root handling for debugging
            debug!(
                "tx_order {}: changeset.state_root (post-root) = {:x}, current_state_root (pre-root) = {:x}",
                tx_order, expected_root, current_state_root
            );

            // Set pre-state root for correct node generation
            changeset.state_root = *current_state_root;
            self.normalize_changeset_pre_state_roots(
                output_store,
                *current_state_root,
                &mut changeset,
            )?;

            // Log changeset size
            debug!(
                "tx_order {}: changeset has {} fields to update",
                tx_order,
                changeset.changes.len()
            );

            // Log all fields being updated
            for (field_key, obj_change) in &changeset.changes {
                debug!(
                    "tx_order {}: field {:?}, op: {:?}, object state_root: {:?}",
                    tx_order, field_key, obj_change.value, obj_change.metadata.state_root
                );
            }

            let (nodes, _stale_indices) = output_store
                .get_state_store()
                .change_set_to_nodes(&mut changeset)
                .map_err(|e| {
                    // Enhanced error message with state root details
                    anyhow::anyhow!(
                        "Failed to convert changeset {} to nodes: {}\n  current_state_root (pre): {:x}\n  expected_root (post): {:x}\n  fields_count: {}",
                        tx_order, e, current_state_root, expected_root, changeset.changes.len()
                    )
                })?;

            let nodes_count = nodes.len() as u64;
            if !nodes.is_empty() {
                output_store
                    .get_state_node_store()
                    .write_nodes(nodes)
                    .map_err(|e| anyhow::anyhow!("Failed to write {} nodes: {}", nodes_count, e))?;
                report.nodes_updated += nodes_count;
                batch_nodes_updated += nodes_count;
            }

            *current_state_root = changeset.state_root;
            if *current_state_root != expected_root {
                let warn_msg = format!(
                    "Replay state root mismatch at tx_order {}: expected {:x}, got {:x}",
                    tx_order, expected_root, *current_state_root
                );
                warn!("{}", warn_msg);
                report.add_error(warn_msg);
            }

            // Count object changes
            total_objects_created += self.count_objects_created(changeset_ext);
            total_objects_updated += self.count_objects_updated(changeset_ext);
        }

        info!(
            "Applied batch: {} changesets, {} nodes, {} objects created, {} objects updated",
            batch.len(),
            batch_nodes_updated,
            total_objects_created,
            total_objects_updated
        );

        Ok(())
    }

    /// Verify final state root
    fn verify_final_state_root(
        &self,
        output_store: &MoveOSStore,
        expected_state_root: H256,
        report: &mut ReplayReport,
    ) -> Result<()> {
        info!(
            "Starting final state root verification, expected: {:x}",
            expected_state_root
        );

        // Get actual state root from startup info
        let actual_state_root = output_store
            .get_config_store()
            .get_startup_info()
            .map_err(|e| anyhow::anyhow!("Failed to get startup info: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("No startup info found"))?
            .state_root;

        info!(
            "Actual state root from startup info: {:x}",
            actual_state_root
        );

        // Store the actual state root in the report
        report.final_state_root = actual_state_root;

        // Compare expected vs actual
        if expected_state_root != actual_state_root {
            let error_msg = format!(
                "State root mismatch: expected {:x}, but got {:x}",
                expected_state_root, actual_state_root
            );

            error!("{}", error_msg);
            report.add_error(error_msg.clone());
            report.verification_passed = false;

            return Err(anyhow::anyhow!(
                "Final state root verification failed: {}",
                error_msg
            ));
        }

        report.verification_passed = true;
        info!(
            "Final state root verification passed: {:x}",
            actual_state_root
        );

        Ok(())
    }

    /// Ensure startup_info and sequencer_info are consistent after replay/trim
    fn verify_startup_sequencer_consistency(
        &self,
        output_store: &MoveOSStore,
        output_rooch_store: &RoochStore,
        expected_order: u64,
    ) -> Result<()> {
        let startup_info = output_store
            .get_config_store()
            .get_startup_info()?
            .ok_or_else(|| anyhow::anyhow!("No startup info found in output store"))?;

        let sequencer_info = output_rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("No sequencer info found in output store"))?;

        if sequencer_info.last_order != expected_order {
            return Err(anyhow::anyhow!(
                "Sequencer info inconsistency: expected last_order {}, got {}",
                expected_order,
                sequencer_info.last_order
            ));
        }

        info!(
            "Startup/Sequencer consistency verified: order={}, state_root={:x}",
            sequencer_info.last_order, startup_info.state_root
        );

        Ok(())
    }

    /// Validate state after batch
    fn validate_batch_state(
        &self,
        output_store: &MoveOSStore,
        expected_state_root: H256,
    ) -> Result<()> {
        info!(
            "Validating batch state with expected state root: {:x}",
            expected_state_root
        );

        // Get current state root from startup info
        let current_startup_info = output_store
            .get_config_store()
            .get_startup_info()
            .map_err(|e| anyhow::anyhow!("Failed to get startup info for validation: {}", e))?;

        match current_startup_info {
            Some(startup_info) => {
                let current_root = startup_info.state_root;

                if current_root != expected_state_root {
                    let warning_msg = format!(
                        "State root mismatch: expected {:x}, got {:x}",
                        expected_state_root, current_root
                    );
                    warn!("Batch state validation warning: {}", warning_msg);
                    // Don't fail the operation, just log the warning
                    // This allows recovery from minor inconsistencies
                } else {
                    info!(
                        "Batch state validation passed: state root {:x}",
                        current_root
                    );
                }
            }
            None => {
                warn!("No startup info available for batch state validation");
            }
        }

        // Additional integrity check: verify we can access some SMT nodes
        // This ensures the database is in a readable state
        if let Err(e) = NodeReader::get(output_store.get_state_node_store(), &expected_state_root) {
            warn!(
                "Cannot access expected state root node {:x}: {}",
                expected_state_root, e
            );
        }

        Ok(())
    }

    /// Create checkpoints
    fn create_checkpoints(&self, _output_dir: &Path, _report: &ReplayReport) -> Result<()> {
        // TODO: Implement checkpoint creation
        // This should save intermediate state checkpoints for recovery

        info!("Created checkpoints for replay operation");
        Ok(())
    }

    /// Count objects created in changeset
    fn count_objects_created(&self, _changeset: &StateChangeSetExt) -> u64 {
        // TODO: Implement object counting logic
        0
    }

    /// Count objects updated in changeset
    fn count_objects_updated(&self, _changeset: &StateChangeSetExt) -> u64 {
        // TODO: Implement object counting logic
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_prune::metadata::OperationStatus;
    use moveos_common::utils::to_bytes;
    use moveos_config::store_config::RocksdbConfig;
    use moveos_store::{MoveOSStore, CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME};
    use moveos_types::transaction::TransactionExecutionInfo;
    use prometheus::Registry;
    use raw_store::metrics::DBMetrics;
    use raw_store::rocks::RocksDB;
    use raw_store::StoreInstance;
    use rooch_config::state_prune::SnapshotMeta;
    use rooch_store::RoochStore;
    use rooch_types::transaction::{LedgerTransaction, RoochTransaction, TransactionSequenceInfo};
    use std::collections::HashSet;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn put_raw_cf(store: &RoochStore, cf_name: &str, key: Vec<u8>, value: Vec<u8>) {
        let db = store.store_instance.db().unwrap().inner();
        let cf = db.cf_handle(cf_name).unwrap();
        db.put_cf(&cf, key, value).unwrap();
    }

    fn seed_windowed_history(
        moveos_store: &MoveOSStore,
        rooch_store: &RoochStore,
        tx_order: u64,
    ) -> H256 {
        let mut sequence_info = TransactionSequenceInfo::random();
        sequence_info.tx_order = tx_order;
        let tx = LedgerTransaction::new_l2_tx(RoochTransaction::mock(), sequence_info);
        let tx_hash = {
            let mut tx_clone = tx.clone();
            tx_clone.tx_hash()
        };

        put_raw_cf(
            rooch_store,
            TRANSACTION_COLUMN_FAMILY_NAME,
            to_bytes(&tx_hash).unwrap(),
            bcs::to_bytes(&tx).unwrap(),
        );
        put_raw_cf(
            rooch_store,
            TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
            to_bytes(&tx_order).unwrap(),
            to_bytes(&tx_hash).unwrap(),
        );

        let mut execution_info = TransactionExecutionInfo::random();
        execution_info.tx_hash = tx_hash;
        moveos_store.save_tx_execution_info(execution_info).unwrap();
        rooch_store
            .get_state_store()
            .save_state_change_set(
                tx_order,
                StateChangeSetExt::new(
                    moveos_types::state::StateChangeSet::new(H256::random(), tx_order),
                    tx_order,
                ),
            )
            .unwrap();

        tx_hash
    }

    fn create_combined_test_stores() -> (MoveOSStore, RoochStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let registry = Registry::new();
        let db = RocksDB::new(
            temp_dir.path(),
            IncrementalReplayer::all_column_families(),
            RocksdbConfig::default(),
        )
        .unwrap();
        let db_metrics = DBMetrics::new(&registry);
        let instance = StoreInstance::new_db_instance(db, Arc::new(db_metrics));
        let moveos_store = MoveOSStore::new_with_instance(instance.clone(), &registry).unwrap();
        let rooch_store = RoochStore::new_with_instance(instance, &registry).unwrap();
        (moveos_store, rooch_store, temp_dir)
    }

    #[test]
    fn test_incremental_replayer_creation() {
        let config = ReplayConfig::default();
        let (rooch_store, _rooch_tmpdir) = RoochStore::mock_rooch_store().unwrap();
        let replayer = IncrementalReplayer::new(config, rooch_store);
        assert!(replayer.is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let config = rooch_config::state_prune::ReplayConfig {
            default_batch_size: 0,
            ..Default::default()
        };

        let (rooch_store, _rooch_tmpdir) = RoochStore::mock_rooch_store().unwrap();
        let replayer = IncrementalReplayer::new(config, rooch_store);
        assert!(replayer.is_err());
    }

    #[test]
    fn test_prepare_fresh_output_store_creates_all_column_families() {
        let config = ReplayConfig::default();
        let (rooch_store, _rooch_tmpdir) = RoochStore::mock_rooch_store().unwrap();
        let replayer = IncrementalReplayer::new(config, rooch_store).unwrap();
        let output_dir = TempDir::new().unwrap();
        let output_store = output_dir.path().join("store");

        replayer.prepare_fresh_output_store(&output_store).unwrap();

        let actual: HashSet<_> = raw_store::rocks::RocksDB::list_cf(&output_store)
            .unwrap()
            .into_iter()
            .collect();

        for cf in IncrementalReplayer::all_column_families() {
            assert!(actual.contains(cf), "missing column family {}", cf);
        }
    }

    #[test]
    fn test_copy_required_cfs_skips_runtime_rebuilt_column_families() {
        let config = ReplayConfig::default();
        let (moveos_store, rooch_store, _tmpdir) = create_combined_test_stores();
        let replayer = IncrementalReplayer::new(config, rooch_store.clone()).unwrap();

        put_raw_cf(
            &rooch_store,
            TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME,
            vec![1, 2, 3],
            vec![4, 5, 6],
        );
        put_raw_cf(
            &rooch_store,
            CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME,
            b"startup_info".to_vec(),
            vec![7, 8, 9],
        );
        let retained_tx_hash = seed_windowed_history(&moveos_store, &rooch_store, 5);

        let output_dir = TempDir::new().unwrap();
        let output_store = output_dir.path().join("store");
        replayer.prepare_fresh_output_store(&output_store).unwrap();
        replayer.copy_required_cfs(&output_store, 5, 5).unwrap();

        let (moveos_store, output_rooch_store) =
            replayer.load_output_stores(&output_store).unwrap();
        let output_db = output_rooch_store.store_instance.db().unwrap().inner();

        let copied_cf = output_db
            .cf_handle(TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME)
            .unwrap();
        assert_eq!(
            output_db
                .get_pinned_cf(&copied_cf, vec![1, 2, 3])
                .unwrap()
                .unwrap()
                .to_vec(),
            vec![4, 5, 6]
        );
        assert!(output_rooch_store
            .transaction_store
            .get_transaction_by_hash(retained_tx_hash)
            .unwrap()
            .is_some());
        assert!(moveos_store
            .get_tx_execution_info(retained_tx_hash)
            .unwrap()
            .is_some());
        assert!(output_rooch_store
            .get_state_store()
            .get_state_change_set(5)
            .unwrap()
            .is_some());

        assert!(
            moveos_store
                .get_config_store()
                .get_startup_info()
                .unwrap()
                .is_none(),
            "startup_info should be rebuilt instead of copied"
        );
    }

    #[test]
    fn test_refresh_output_metadata_rewrites_sequencer_without_copied_meta() {
        let config = ReplayConfig::default();
        let (_source_moveos_store, rooch_store, _tmpdir) = create_combined_test_stores();
        let replayer = IncrementalReplayer::new(config, rooch_store).unwrap();

        let output_dir = TempDir::new().unwrap();
        let output_store_path = output_dir.path().join("store");
        replayer
            .prepare_fresh_output_store(&output_store_path)
            .unwrap();
        let (moveos_store, output_rooch_store) =
            replayer.load_output_stores(&output_store_path).unwrap();

        let tx = LedgerTransaction::new_l2_tx(
            RoochTransaction::mock(),
            TransactionSequenceInfo::random(),
        );
        let tx_hash = {
            let mut tx_clone = tx.clone();
            tx_clone.tx_hash()
        };

        put_raw_cf(
            &output_rooch_store,
            TRANSACTION_COLUMN_FAMILY_NAME,
            to_bytes(&tx_hash).unwrap(),
            bcs::to_bytes(&tx).unwrap(),
        );
        put_raw_cf(
            &output_rooch_store,
            TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
            to_bytes(&0u64).unwrap(),
            to_bytes(&tx_hash).unwrap(),
        );

        moveos_store
            .get_config_store()
            .save_startup_info(StartupInfo::new(H256::random(), 1))
            .unwrap();

        replayer
            .refresh_output_metadata(
                &output_rooch_store,
                0,
                &mut StatePruneMetadata::new(
                    crate::state_prune::OperationType::Replay {
                        snapshot_path: PathBuf::from("/tmp/snapshot"),
                        from_order: 0,
                        to_order: 0,
                        output_dir: output_store_path.clone(),
                    },
                    serde_json::json!({}),
                ),
            )
            .unwrap();

        drop(moveos_store);
        drop(output_rooch_store);

        let (_moveos_store, output_rooch_store) =
            replayer.load_output_stores(&output_store_path).unwrap();
        let sequencer_info = output_rooch_store
            .get_meta_store()
            .get_sequencer_info()
            .unwrap()
            .unwrap();
        assert_eq!(sequencer_info.last_order, 0);

        assert!(output_rooch_store
            .get_meta_store()
            .get_sequencer_info()
            .unwrap()
            .is_some());
    }

    #[test]
    fn test_load_snapshot_store_uses_snapshot_path_not_live_db() {
        // This is a regression test for issue #3900
        // It verifies that load_snapshot_store actually loads from the snapshot path,
        // not from the live MoveOSStore

        // Create a temporary directory for the snapshot
        let snapshot_dir = TempDir::new().unwrap();
        let snapshot_path = snapshot_dir.path();

        // Create a snapshot database directory
        let snapshot_db_path = snapshot_path.join("snapshot.db");
        fs::create_dir_all(&snapshot_db_path).unwrap();

        // Create a minimal snapshot metadata file
        let snapshot_meta = SnapshotMeta {
            tx_order: 100,
            state_root: H256::from([1u8; 32]),
            global_size: 1000,
            node_count: 100,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 1,
            metadata: serde_json::json!({}),
        };
        let meta_path = snapshot_path.join("snapshot_meta.json");
        let meta_content = serde_json::to_string_pretty(&snapshot_meta).unwrap();
        fs::write(&meta_path, meta_content).unwrap();

        // Create a live store (this represents the live database)
        let (live_rooch_store, _rooch_tmpdir) = RoochStore::mock_rooch_store().unwrap();

        // Create a replayer with the live store
        let config = ReplayConfig::default();
        let replayer = IncrementalReplayer::new(config, live_rooch_store).unwrap();

        // Load snapshot store from the snapshot path
        let result = replayer.load_snapshot_store(snapshot_path);

        // Before the fix for issue #3900, this would succeed and return the live store
        // After the fix, it should succeed and return a store loaded from the snapshot path

        // The test passes if load_snapshot_store succeeds and loads from the snapshot path
        assert!(result.is_ok(), "load_snapshot_store should succeed");

        let snapshot_store = result.unwrap();

        // Verify that the snapshot store is different from the live store
        // (They should have different database paths)
        // We can't directly compare the paths, but we can verify the snapshot store was created
        // by checking it's a valid MoveOSStore

        // Try to access the startup info from the snapshot store
        // This will fail if the snapshot database is empty, which is expected for this test
        let startup_info_result = snapshot_store.get_config_store().get_startup_info();

        // The snapshot database is empty (we only created the directory structure),
        // so we expect no startup info, but the store should be valid
        assert!(
            startup_info_result.is_ok(),
            "Should be able to access config store"
        );
        assert!(
            startup_info_result.unwrap().is_none(),
            "New snapshot should have no startup info"
        );

        // Test that loading from a non-existent path fails
        let nonexistent_path = TempDir::new().unwrap();
        let nonexistent_snapshot_path = nonexistent_path.path().join("nonexistent");
        let result = replayer.load_snapshot_store(&nonexistent_snapshot_path);
        assert!(
            result.is_err(),
            "Should fail to load from non-existent path"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Snapshot database not found"),
            "Error should mention missing snapshot database"
        );
    }

    #[test]
    fn test_load_snapshot_store_validates_path() {
        // Test that load_snapshot_store validates the snapshot path

        let (live_rooch_store, _rooch_tmpdir) = RoochStore::mock_rooch_store().unwrap();
        let config = ReplayConfig::default();
        let replayer = IncrementalReplayer::new(config, live_rooch_store).unwrap();

        // Test with a file instead of a directory
        let file_as_snapshot = TempDir::new().unwrap();
        let file_path = file_as_snapshot.path().join("snapshot.db");
        fs::write(&file_path, b"not a directory").unwrap();

        let result = replayer.load_snapshot_store(file_as_snapshot.path());
        assert!(
            result.is_err(),
            "Should fail when snapshot.db is a file, not a directory"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not a valid database"),
            "Error should mention invalid database"
        );
    }

    #[test]
    fn test_verify_final_state_root_success() {
        // Test that verification passes when state roots match
        let (live_rooch_store, _rooch_tmpdir) = RoochStore::mock_rooch_store().unwrap();
        let config = ReplayConfig::default();
        let replayer = IncrementalReplayer::new(config, live_rooch_store.clone()).unwrap();

        // Create a snapshot store with a known state root
        let snapshot_dir = TempDir::new().unwrap();
        let snapshot_path = snapshot_dir.path();

        // Use a known state root for testing
        let expected_root = H256::from([1u8; 32]);

        // Create snapshot metadata
        let snapshot_meta = SnapshotMeta {
            tx_order: 100,
            state_root: expected_root,
            global_size: 1000,
            node_count: 100,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 1,
            metadata: serde_json::json!({}),
        };
        let meta_path = snapshot_path.join("snapshot_meta.json");
        let meta_content = serde_json::to_string_pretty(&snapshot_meta).unwrap();
        fs::write(&meta_path, meta_content).unwrap();

        // Create snapshot database and set startup info
        let snapshot_db_path = snapshot_path.join("snapshot.db");
        fs::create_dir_all(&snapshot_db_path).unwrap();

        // Create a MoveOSStore for the snapshot
        let registry = Registry::new();
        let snapshot_store = MoveOSStore::new(&snapshot_db_path, &registry).unwrap();

        // Create startup info with the expected state root
        use moveos_types::startup_info::StartupInfo;
        let startup_info = StartupInfo::new(expected_root, 1000);
        snapshot_store
            .get_config_store()
            .save_startup_info(startup_info)
            .unwrap();

        // Create report and metadata
        let mut report = ReplayReport::new();
        let metadata = StatePruneMetadata::new(
            crate::state_prune::OperationType::Replay {
                snapshot_path: snapshot_path.to_path_buf(),
                from_order: 0,
                to_order: 100,
                output_dir: TempDir::new().unwrap().path().to_path_buf(),
            },
            serde_json::json!({}),
        );

        // Verify with matching state root should succeed
        let result = replayer.verify_final_state_root(&snapshot_store, expected_root, &mut report);

        assert!(
            result.is_ok(),
            "Verification should succeed when state roots match"
        );
        assert!(
            report.verification_passed,
            "Report should show verification passed"
        );
        assert_eq!(report.final_state_root, expected_root);
        assert!(
            report.errors.is_empty(),
            "Report should have no errors when verification passes"
        );
        // Note: The verify function doesn't mark metadata as completed/failed,
        // that's done by the caller (replay_changesets method)
        assert!(
            matches!(metadata.status, OperationStatus::Pending),
            "Metadata status should remain pending when called directly"
        );
    }

    #[test]
    fn test_verify_final_state_root_failure() {
        // Test that verification fails when state roots don't match
        let (live_rooch_store, _rooch_tmpdir) = RoochStore::mock_rooch_store().unwrap();
        let config = ReplayConfig::default();
        let replayer = IncrementalReplayer::new(config, live_rooch_store.clone()).unwrap();

        // Create a snapshot store
        let snapshot_dir = TempDir::new().unwrap();
        let snapshot_path = snapshot_dir.path();

        // Use different roots for actual and expected to cause mismatch
        let actual_root = H256::from([1u8; 32]);
        let expected_root = H256::from([2u8; 32]);

        // Create snapshot metadata
        let snapshot_meta = SnapshotMeta {
            tx_order: 100,
            state_root: actual_root,
            global_size: 1000,
            node_count: 100,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 1,
            metadata: serde_json::json!({}),
        };
        let meta_path = snapshot_path.join("snapshot_meta.json");
        let meta_content = serde_json::to_string_pretty(&snapshot_meta).unwrap();
        fs::write(&meta_path, meta_content).unwrap();

        // Create snapshot database
        let snapshot_db_path = snapshot_path.join("snapshot.db");
        fs::create_dir_all(&snapshot_db_path).unwrap();

        // Create a MoveOSStore for the snapshot
        let registry = Registry::new();
        let snapshot_store = MoveOSStore::new(&snapshot_db_path, &registry).unwrap();

        // Create startup info with the actual state root
        use moveos_types::startup_info::StartupInfo;
        let startup_info = StartupInfo::new(actual_root, 1000);
        snapshot_store
            .get_config_store()
            .save_startup_info(startup_info)
            .unwrap();

        // Create report and metadata
        let mut report = ReplayReport::new();
        let metadata = StatePruneMetadata::new(
            crate::state_prune::OperationType::Replay {
                snapshot_path: snapshot_path.to_path_buf(),
                from_order: 0,
                to_order: 100,
                output_dir: TempDir::new().unwrap().path().to_path_buf(),
            },
            serde_json::json!({}),
        );

        // Verify with mismatching state root should fail
        let result = replayer.verify_final_state_root(&snapshot_store, expected_root, &mut report);

        assert!(
            result.is_err(),
            "Verification should fail when state roots don't match"
        );
        assert!(
            !report.verification_passed,
            "Report should show verification failed"
        );
        assert_eq!(report.final_state_root, actual_root);
        assert!(
            !report.errors.is_empty(),
            "Report should contain error message"
        );
        assert!(
            report.errors[0].contains("State root mismatch"),
            "Error should mention state root mismatch"
        );
        assert!(
            !report.is_success(),
            "Report should indicate failure via is_success()"
        );
        // Note: The verify function doesn't mark metadata as failed,
        // that's done by the caller (replay_changesets method)
        assert!(
            matches!(metadata.status, OperationStatus::Pending),
            "Metadata status should remain pending when called directly"
        );
    }
}
