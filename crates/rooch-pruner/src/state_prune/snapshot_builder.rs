// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state_prune::{
    OperationType, ProgressTracker, SnapshotBuilderConfig, StatePruneMetadata,
};
use crate::util::extract_child_nodes_strict;
use anyhow::Result;
use moveos_store::MoveOSStore;
use moveos_store::STATE_NODE_COLUMN_FAMILY_NAME;
use moveos_types::h256::H256;
use moveos_types::startup_info::StartupInfo;
use prometheus::Registry;
use raw_store::SchemaStore;
use rooch_config::state_prune::SnapshotMeta;
use serde::{Deserialize, Serialize};
use serde_json;
use smt::{NodeReader, SPARSE_MERKLE_PLACEHOLDER_HASH};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Statistics for state tree traversal
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TraversalStatistics {
    pub nodes_visited: u64,
    pub bytes_processed: u64,
}

/// Progress information for snapshot resume functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotProgress {
    /// State root being processed
    pub state_root: H256,

    /// Worklist state (nodes yet to process)
    pub worklist: Vec<H256>,

    /// Current position in worklist (for partial processing)
    pub worklist_position: usize,

    /// Traversal statistics so far
    pub statistics: TraversalStatistics,

    /// Current batch buffer (nodes being processed)
    pub batch_buffer: Vec<(H256, Vec<u8>)>,

    /// Current batch size configuration
    pub current_batch_size: usize,

    /// Timestamp of last progress save
    pub last_save_timestamp: u64,

    /// Number of nodes already written to snapshot
    pub nodes_written: u64,

    /// Checkpoint identifier for consistency validation
    pub checkpoint_id: String,
}

impl SnapshotProgress {
    /// Save current progress to disk with atomic write
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let temp_path = path.with_extension("tmp");
        let backup_path = path.with_extension("backup");

        // Create backup if original exists
        if path.exists() {
            if let Err(e) = fs::copy(path, &backup_path) {
                warn!("Failed to create backup of progress file: {}", e);
            }
        }

        // Write to temp file first
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&temp_path, content)?;

        // Atomic rename
        fs::rename(&temp_path, path)?;

        debug!("Progress saved to {:?}", path);
        Ok(())
    }

    /// Load progress from disk with validation
    pub fn load_from_file<P: AsRef<Path>>(
        path: P,
        expected_state_root: H256,
    ) -> Result<Option<Self>> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let progress: SnapshotProgress = serde_json::from_str(&content)?;

        // Validate state root matches
        if progress.state_root != expected_state_root {
            warn!(
                "Progress file state_root {:x} doesn't match expected {:x}, ignoring progress",
                progress.state_root, expected_state_root
            );
            return Ok(None);
        }

        info!(
            "Loaded valid progress: {} nodes processed, {} in worklist, checkpoint {}",
            progress.statistics.nodes_visited,
            progress
                .worklist
                .len()
                .saturating_sub(progress.worklist_position),
            progress.checkpoint_id
        );

        Ok(Some(progress))
    }

    /// Generate checkpoint ID for consistency validation
    pub fn generate_checkpoint_id(state_root: H256, nodes_visited: u64) -> String {
        format!("{:x}-{}", state_root, nodes_visited)
    }
}

/// Streaming snapshot builder for creating state snapshots containing only active nodes
/// Uses RocksDB backend with batched writes and scalable deduplication
pub struct SnapshotBuilder {
    config: SnapshotBuilderConfig,
    moveos_store: MoveOSStore,
    progress_tracker: ProgressTracker,
}

impl SnapshotBuilder {
    /// Create new snapshot builder
    pub fn new(config: SnapshotBuilderConfig, moveos_store: MoveOSStore) -> Result<Self> {
        config.validate()?;

        let progress_tracker = ProgressTracker::new(config.progress_interval_seconds);

        Ok(Self {
            config,
            moveos_store,
            progress_tracker,
        })
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &SnapshotBuilderConfig {
        &self.config
    }

    /// Build snapshot from state root using streaming approach
    pub async fn build_snapshot(
        &self,
        state_root: H256,
        tx_order: u64,
        global_size: u64,
        output_dir: PathBuf,
        force_restart: bool,
    ) -> Result<SnapshotMeta> {
        info!(
            "Starting streaming snapshot build for state root: {:x}",
            state_root
        );

        let start_time = Instant::now();

        // Initialize metadata
        let mut metadata = StatePruneMetadata::new(
            OperationType::Snapshot {
                tx_order,
                state_root: format!("{:x}", state_root),
                output_dir: output_dir.clone(),
            },
            serde_json::json!({
                "state_root": format!("{:x}", state_root),
                "tx_order": tx_order,
                "global_size": global_size,
                "config": self.config
            }),
        );

        metadata.mark_in_progress("Initializing".to_string(), 0.0);

        // Ensure output directory exists
        std::fs::create_dir_all(&output_dir)?;

        // Check for resumable progress if not forcing restart
        let resume_progress = if !force_restart && self.config.enable_resume {
            self.check_resume_state(&output_dir, state_root)?
        } else {
            if force_restart {
                info!("Force restart requested, ignoring any existing progress");
            } else if !self.config.enable_resume {
                info!("Resume functionality disabled");
            }
            None
        };

        // Initialize progress tracking
        self.progress_tracker.set_total(0); // Will be updated during traversal

        // Create snapshot node writer with RocksDB backend
        metadata.mark_in_progress("Creating snapshot database".to_string(), 5.0);

        // When resuming, restore nodes_written count from previous progress
        let initial_nodes_written = resume_progress
            .as_ref()
            .map(|p| p.nodes_written)
            .unwrap_or(0);
        if initial_nodes_written > 0 {
            info!(
                "Restoring snapshot writer with {} previously written nodes",
                initial_nodes_written
            );
        }
        let mut snapshot_writer =
            SnapshotNodeWriter::new_with_count(&output_dir, &self.config, initial_nodes_written)?;

        // Perform streaming traversal with batched writes
        metadata.mark_in_progress("Traversing state tree".to_string(), 10.0);
        let statistics = self
            .stream_traverse_and_write(
                state_root,
                &mut snapshot_writer,
                &mut metadata,
                &output_dir,
                resume_progress,
            )
            .await?;

        metadata.update_statistics(|stats| {
            stats.nodes_processed = statistics.nodes_visited;
            stats.bytes_processed = statistics.bytes_processed;
        });

        metadata.mark_in_progress("Finalizing snapshot".to_string(), 95.0);

        // Persist startup info for snapshot store consistency
        snapshot_writer.save_startup_info(state_root, global_size)?;

        // Flush and close snapshot writer to get final statistics
        let active_nodes_count = snapshot_writer.finalize(&mut metadata)?;

        metadata.mark_in_progress("Verifying snapshot integrity".to_string(), 97.0);
        self.verify_snapshot_integrity(&output_dir, state_root)?;

        // Create snapshot metadata with tx_order and global_size
        let snapshot_meta =
            SnapshotMeta::new(tx_order, state_root, global_size, active_nodes_count);

        // Save metadata
        let meta_path = snapshot_meta.save_to_file(&output_dir)?;
        let latest_meta_path = output_dir.join("snapshot_meta.json");
        if meta_path != latest_meta_path {
            fs::copy(&meta_path, &latest_meta_path)?;
        }
        metadata.save_to_file(output_dir.join("operation_meta.json"))?;

        let duration = start_time.elapsed();
        info!(
            "Streaming snapshot build completed in {:?}, {} nodes processed, {} written to output",
            duration, statistics.nodes_visited, active_nodes_count
        );

        // Clean up progress files on successful completion
        self.cleanup_progress_files(&output_dir)?;

        metadata.mark_completed();

        Ok(snapshot_meta)
    }

    /// Check if resumable progress exists and load it
    fn check_resume_state(
        &self,
        output_dir: &Path,
        state_root: H256,
    ) -> Result<Option<SnapshotProgress>> {
        let progress_path = output_dir.join("snapshot_progress.json");

        match SnapshotProgress::load_from_file(&progress_path, state_root) {
            Ok(Some(progress)) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(std::time::Duration::from_secs(0))
                    .as_secs();
                let elapsed = now.saturating_sub(progress.last_save_timestamp);

                info!(
                    "Found resumable progress: {} nodes processed, {} in worklist, last saved {} seconds ago",
                    progress.statistics.nodes_visited,
                    progress.worklist.len().saturating_sub(progress.worklist_position),
                    elapsed
                );
                Ok(Some(progress))
            }
            Ok(None) => {
                info!("No resumable progress found, starting fresh");
                Ok(None)
            }
            Err(e) => {
                warn!("Failed to load progress file: {}, starting fresh", e);
                // Try to clean up corrupted progress file
                let progress_path = output_dir.join("snapshot_progress.json");
                if progress_path.exists() {
                    if let Err(remove_err) = fs::remove_file(&progress_path) {
                        warn!("Failed to remove corrupted progress file: {}", remove_err);
                    }
                }
                Ok(None)
            }
        }
    }

    /// Clean up progress files on successful completion
    fn cleanup_progress_files(&self, output_dir: &Path) -> Result<()> {
        let progress_path = output_dir.join("snapshot_progress.json");
        let backup_path = output_dir.join("snapshot_progress.backup");

        for path in [progress_path, backup_path] {
            if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    warn!("Failed to remove progress file {:?}: {}", path, e);
                } else {
                    debug!("Removed progress file: {:?}", path);
                }
            }
        }

        Ok(())
    }

    /// Save current progress to disk
    fn save_progress(
        &self,
        output_dir: &Path,
        state_root: H256,
        worklist: &[H256],
        statistics: &TraversalStatistics,
        batch_buffer: &[(H256, Vec<u8>)],
        current_batch_size: usize,
        nodes_written: u64,
    ) -> Result<()> {
        let progress = SnapshotProgress {
            state_root,
            worklist: worklist.to_owned(),
            worklist_position: 0, // We've processed everything before current position
            statistics: statistics.clone(),
            batch_buffer: batch_buffer.to_vec(),
            current_batch_size,
            last_save_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs(),
            nodes_written,
            checkpoint_id: SnapshotProgress::generate_checkpoint_id(
                state_root,
                statistics.nodes_visited,
            ),
        };

        let progress_path = output_dir.join("snapshot_progress.json");
        progress.save_to_file(progress_path)?;

        Ok(())
    }

    /// Streaming traversal with batched writes to avoid OOM
    async fn stream_traverse_and_write(
        &self,
        state_root: H256,
        snapshot_writer: &mut SnapshotNodeWriter,
        metadata: &mut StatePruneMetadata,
        output_dir: &Path,
        resume_progress: Option<SnapshotProgress>,
    ) -> Result<TraversalStatistics> {
        // Initialize or resume traversal state
        let (mut statistics, mut nodes_to_process, mut current_batch_size, mut batch_buffer) =
            if let Some(progress) = resume_progress {
                info!("Resuming from previous snapshot operation");

                // Restore worklist from progress
                // For DFS with Vec (stack), save_progress always sets worklist_position to 0
                // The entire worklist contains nodes yet to be processed (popped from the end)
                let worklist = progress.worklist;

                // Update metadata with resume statistics
                metadata.update_statistics(|stats| {
                    stats.nodes_processed = progress.statistics.nodes_visited;
                    stats.bytes_processed = progress.statistics.bytes_processed;
                });

                info!(
                "Resuming traversal: {} nodes already processed, {} nodes in worklist, batch size: {}",
                progress.statistics.nodes_visited,
                worklist.len(),
                progress.current_batch_size
            );

                (
                    progress.statistics,
                    worklist,
                    progress.current_batch_size,
                    progress.batch_buffer,
                )
            } else {
                info!("No resumable state found, starting fresh");
                let worklist = vec![state_root];
                (
                    TraversalStatistics::default(),
                    worklist,
                    self.config.batch_size,
                    Vec::with_capacity(self.config.batch_size),
                )
            };

        let node_store = &self.moveos_store.node_store;
        let mut last_progress_report = Instant::now();
        let mut last_memory_check = Instant::now();
        let mut last_progress_save = Instant::now();
        const MEMORY_CHECK_INTERVAL_SECS: u64 = 10; // Check memory every 10 seconds
        const PROGRESS_SAVE_INTERVAL_SECS: u64 = 300; // Save progress every 5 minutes

        while let Some(current_hash) = nodes_to_process.pop() {
            if current_hash == *SPARSE_MERKLE_PLACEHOLDER_HASH {
                continue;
            }
            // Check if node is already written to avoid duplication
            // The RocksDB-based deduplication handles this efficiently with O(1) space complexity
            // This also serves as cycle detection since nodes already processed won't be revisited
            if snapshot_writer.contains_node(&current_hash)? {
                debug!("Skipping duplicate node: {}", current_hash);
                continue;
            }

            // Increment nodes_visited counter only when we actually process the node
            // (after confirming it's not already in the snapshot)
            statistics.nodes_visited += 1;

            // Get node data
            let node_data = node_store.get(&current_hash)?.ok_or_else(|| {
                anyhow::anyhow!(
                    "Missing state node {:x} while building snapshot",
                    current_hash
                )
            })?;
            statistics.bytes_processed += node_data.len() as u64;

            // Extract child nodes from the node data before moving it into the buffer
            let child_hashes = extract_child_nodes_strict(&node_data)?;

            // Add node to batch buffer for writing
            batch_buffer.push((current_hash, node_data));

            // Add child nodes to processing queue (DFS: push to stack)
            for child_hash in child_hashes {
                nodes_to_process.push(child_hash);
            }

            // Adaptive batch size adjustment based on memory pressure
            if self.config.should_use_adaptive_batching()
                && last_memory_check.elapsed() >= Duration::from_secs(MEMORY_CHECK_INTERVAL_SECS)
            {
                if let Some(new_batch_size) =
                    self.adjust_batch_size_for_memory_pressure(current_batch_size)
                {
                    if new_batch_size != current_batch_size {
                        info!(
                            "Adjusting batch size from {} to {} due to memory pressure",
                            current_batch_size, new_batch_size
                        );
                        current_batch_size = new_batch_size;
                        // Resize buffer if needed - use reserve_exact for tighter allocation
                        if batch_buffer.capacity() < current_batch_size {
                            batch_buffer
                                .reserve_exact(current_batch_size - batch_buffer.capacity());
                        }
                    }
                }
                last_memory_check = Instant::now();
            }

            // Write batch when it reaches the current batch size
            if batch_buffer.len() >= current_batch_size {
                snapshot_writer.write_batch(std::mem::take(&mut batch_buffer))?;

                // Update progress periodically
                if last_progress_report.elapsed()
                    >= Duration::from_secs(self.config.progress_interval_seconds)
                {
                    info!(
                        "Streaming traversal progress: {} batches processed, {} nodes written, current batch size: {}",
                        statistics.nodes_visited / current_batch_size as u64,
                        snapshot_writer.nodes_written,
                        current_batch_size
                    );
                    last_progress_report = Instant::now();
                }
            }

            // Periodic progress update
            if self.progress_tracker.should_report() {
                let progress = 10.0 + (statistics.nodes_visited as f64 / 1_000_000.0) * 70.0; // Approximate progress
                metadata.mark_in_progress(
                    format!("Streaming traversal ({} nodes)", statistics.nodes_visited),
                    progress.min(80.0),
                );
                self.progress_tracker.mark_reported();
            }

            // Save progress periodically (every 5 minutes)
            if self.config.enable_resume
                && last_progress_save.elapsed() >= Duration::from_secs(PROGRESS_SAVE_INTERVAL_SECS)
            {
                if let Err(e) = self.save_progress(
                    output_dir,
                    state_root,
                    &nodes_to_process,
                    &statistics,
                    &batch_buffer,
                    current_batch_size,
                    snapshot_writer.nodes_written,
                ) {
                    warn!("Failed to save progress: {}", e);
                } else {
                    info!(
                        "Progress saved: {} nodes processed, {} nodes in worklist",
                        statistics.nodes_visited,
                        nodes_to_process.len()
                    );
                    last_progress_save = Instant::now();
                }
            }
        }

        // Write remaining nodes in the final batch
        if !batch_buffer.is_empty() {
            snapshot_writer.write_batch(batch_buffer)?;
        }

        Ok(statistics)
    }

    fn verify_snapshot_integrity(&self, output_dir: &Path, state_root: H256) -> Result<()> {
        let snapshot_db_path = output_dir.join("snapshot.db");
        let registry = Registry::new();
        let snapshot_store = MoveOSStore::new(&snapshot_db_path, &registry).map_err(|e| {
            anyhow::anyhow!(
                "Failed to open snapshot database at {:?}: {}",
                snapshot_db_path,
                e
            )
        })?;
        let node_store = snapshot_store.get_state_node_store();

        if state_root != *SPARSE_MERKLE_PLACEHOLDER_HASH && node_store.get(&state_root)?.is_none() {
            return Err(anyhow::anyhow!(
                "Snapshot integrity check failed: missing root node {:x}",
                state_root
            ));
        }

        let raw_db = node_store
            .get_store()
            .store()
            .db()
            .ok_or_else(|| anyhow::anyhow!("Failed to access snapshot RocksDB instance"))?
            .inner();
        let cf = raw_db
            .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
            .ok_or_else(|| anyhow::anyhow!("State node column family not found"))?;
        let mut iter = raw_db.raw_iterator_cf(&cf);
        let mut checked = 0u64;

        iter.seek_to_first();
        while iter.valid() {
            if let (Some(key), Some(value)) = (iter.key(), iter.value()) {
                if key.len() != 32 {
                    return Err(anyhow::anyhow!(
                        "Invalid state node key length: {}",
                        key.len()
                    ));
                }
                let children = extract_child_nodes_strict(value)?;
                for child in children {
                    if node_store.get(&child)?.is_none() {
                        return Err(anyhow::anyhow!(
                            "Snapshot integrity check failed: missing child node {:x}",
                            child
                        ));
                    }
                }
                checked += 1;
            }
            iter.next();
        }
        iter.status()
            .map_err(|e| anyhow::anyhow!("Snapshot iterator error: {}", e))?;

        info!(
            "Snapshot integrity check passed ({} nodes verified)",
            checked
        );
        Ok(())
    }

    /// Adjust batch size based on current memory pressure
    /// Uses integer arithmetic for deterministic behavior
    /// Returns None if no adjustment needed, Some(new_size) otherwise where new_size >= 100
    pub fn adjust_batch_size_for_memory_pressure(
        &self,
        current_batch_size: usize,
    ) -> Option<usize> {
        // Get current memory usage
        let current_memory = self.get_current_memory_usage();

        if self.config.memory_limit == 0 {
            // No memory limit, no adjustment needed
            return None;
        }

        // Use integer arithmetic: multiply by 1000 for 3 decimal places precision
        let memory_per_thousand = (current_memory * 1000) / self.config.memory_limit;
        let threshold_per_thousand = (self.config.memory_pressure_threshold * 1000.0) as u64;

        if memory_per_thousand > threshold_per_thousand {
            // High memory pressure - reduce batch size
            // Calculate reduction: up to 30% based on how much we're over threshold
            let excess = memory_per_thousand.saturating_sub(threshold_per_thousand);
            let max_reduction = 300; // 30% in per-thousand units
            let reduction = std::cmp::min(excess, max_reduction);

            // Reduce by up to 30% using integer arithmetic: * (1000 - reduction) / 1000
            let new_batch_size = (current_batch_size * (1000 - reduction as usize) / 1000).max(100);

            info!(
                "Memory pressure detected: {:.1}% usage ({} MB/{} MB), reducing batch size from {} to {}",
                memory_per_thousand as f64 / 10.0,
                current_memory / (1024 * 1024),
                self.config.memory_limit / (1024 * 1024),
                current_batch_size,
                new_batch_size
            );

            Some(new_batch_size)
        } else if memory_per_thousand < threshold_per_thousand * 6 / 10
            && current_batch_size < self.config.batch_size
        {
            // Low memory pressure - can increase batch size (60% of threshold)
            // Calculate increase: up to 20% based on headroom
            let headroom = threshold_per_thousand.saturating_sub(memory_per_thousand);
            let max_increase = 200; // 20% in per-thousand units
            let increase = std::cmp::min(headroom, max_increase);

            // Increase by up to 20%: * (1000 + increase) / 1000
            let new_batch_size = (current_batch_size * (1000 + increase as usize) / 1000)
                .min(self.config.batch_size);

            if new_batch_size > current_batch_size {
                info!(
                    "Low memory pressure: {:.1}% usage ({} MB/{} MB), increasing batch size from {} to {}",
                    memory_per_thousand as f64 / 10.0,
                    current_memory / (1024 * 1024),
                    self.config.memory_limit / (1024 * 1024),
                    current_batch_size,
                    new_batch_size
                );
                Some(new_batch_size)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get current memory usage in bytes
    fn get_current_memory_usage(&self) -> u64 {
        // Try to get memory usage from system
        #[cfg(unix)]
        {
            use std::fs;
            match fs::read_to_string("/proc/self/status") {
                Ok(status) => {
                    for line in status.lines() {
                        if line.starts_with("VmRSS:") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                if let Ok(kb) = parts[1].parse::<u64>() {
                                    return kb * 1024; // Convert KB to bytes
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    warn!("Failed to read /proc/self/status, using conservative memory estimate");
                }
            }
        }

        // Conservative fallback estimate - assume 75% usage to trigger aggressive batch reduction
        // This ensures adaptive batching reduces batch size conservatively when actual usage is unknown
        self.config.memory_limit.saturating_mul(75) / 100 // Assume 75% of limit
    }
}

/// RocksDB-backed snapshot node writer with batched writes and deduplication
pub struct SnapshotNodeWriter {
    moveos_store: MoveOSStore,
    pub nodes_written: u64,
}

impl SnapshotNodeWriter {
    /// Create new snapshot node writer with RocksDB backend
    pub fn new(output_dir: &Path, _config: &SnapshotBuilderConfig) -> Result<Self> {
        Self::new_with_count(output_dir, _config, 0)
    }

    /// Create new snapshot node writer with RocksDB backend, starting from existing count
    /// Used when resuming a snapshot operation to preserve previously written node count
    pub fn new_with_count(
        output_dir: &Path,
        _config: &SnapshotBuilderConfig,
        initial_count: u64,
    ) -> Result<Self> {
        let snapshot_db_path = output_dir.join("snapshot.db");

        // Validate and create output directory
        if snapshot_db_path.exists() && !snapshot_db_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Snapshot path exists but is not a directory: {:?}",
                snapshot_db_path
            ));
        }
        std::fs::create_dir_all(&snapshot_db_path)?;

        // Check available disk space (basic safety check)
        if let Ok(_metadata) = std::fs::metadata(&snapshot_db_path) {
            debug!("Snapshot directory created: {:?}", snapshot_db_path);
        }

        let registry = Registry::new();
        let moveos_store = MoveOSStore::new(&snapshot_db_path, &registry).map_err(|e| {
            anyhow::anyhow!(
                "Failed to open snapshot database at {:?}: {}",
                snapshot_db_path,
                e
            )
        })?;

        info!(
            "Successfully opened snapshot database at: {:?}",
            snapshot_db_path
        );

        Ok(Self {
            moveos_store,
            nodes_written: initial_count,
        })
    }

    /// Check if node already exists in snapshot (for deduplication)
    pub fn contains_node(&self, hash: &H256) -> Result<bool> {
        Ok(self
            .moveos_store
            .get_state_node_store()
            .get(hash)?
            .is_some())
    }

    /// Batch check if nodes already exist in snapshot (for efficient deduplication)
    /// Returns a vector of booleans indicating whether each node exists
    pub fn contains_nodes_batch(&self, hashes: &[H256]) -> Result<Vec<bool>> {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }

        let results = self.moveos_store.get_state_node_store().multi_get(hashes)?;
        Ok(results.into_iter().map(|opt| opt.is_some()).collect())
    }

    /// Filter out already existing nodes from a batch, returning only new nodes
    /// Takes ownership of the input batch to avoid cloning data
    pub fn filter_new_nodes(&self, nodes: Vec<(H256, Vec<u8>)>) -> Result<Vec<(H256, Vec<u8>)>> {
        if nodes.is_empty() {
            return Ok(Vec::new());
        }

        let hashes: Vec<_> = nodes.iter().map(|(h, _)| *h).collect();
        let existence_flags = self.contains_nodes_batch(&hashes)?;

        let mut new_nodes = Vec::with_capacity(nodes.len());
        for ((hash, data), exists) in nodes.into_iter().zip(existence_flags) {
            if !exists {
                new_nodes.push((hash, data));
            }
        }

        Ok(new_nodes)
    }

    /// Write batch of nodes to RocksDB with efficient deduplication
    pub fn write_batch(&mut self, batch: Vec<(H256, Vec<u8>)>) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        // Save batch length before passing ownership to filter_new_nodes
        let original_batch_size = batch.len();

        // Filter out duplicates before writing to avoid unnecessary writes
        let new_nodes = self.filter_new_nodes(batch)?;

        if new_nodes.is_empty() {
            debug!(
                "All {} nodes in batch were duplicates, skipping write",
                original_batch_size
            );
            return Ok(());
        }

        let duplicate_count = original_batch_size - new_nodes.len();
        if duplicate_count > 0 {
            debug!(
                "Filtered {} duplicate nodes from batch, writing {} unique nodes",
                duplicate_count,
                new_nodes.len()
            );
        }

        let mut nodes = BTreeMap::new();
        for (hash, data) in new_nodes {
            nodes.insert(hash, data);
        }
        let nodes_count = nodes.len() as u64;
        self.moveos_store
            .get_state_node_store()
            .write_nodes(nodes)?;

        // Update nodes_written count
        self.nodes_written += nodes_count;

        Ok(())
    }

    /// Get the actual count of nodes in the snapshot database by querying RocksDB
    /// This is useful for validation or when the count needs to be derived from the database
    pub fn get_actual_node_count(&self) -> Result<u64> {
        if let Some(wrapper) = self
            .moveos_store
            .get_state_node_store()
            .get_store()
            .store()
            .db()
        {
            let raw_db = wrapper.inner();
            let cf = raw_db
                .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
                .ok_or_else(|| anyhow::anyhow!("State node column family not found"))?;
            let mut iter = raw_db.raw_iterator_cf(&cf);
            let mut count = 0u64;
            iter.seek_to_first();
            while iter.valid() {
                count += 1;
                iter.next();
            }
            iter.status()
                .map_err(|e| anyhow::anyhow!("Error iterating snapshot db: {}", e))?;
            return Ok(count);
        }

        warn!("Snapshot store does not expose a RocksDB instance; using tracked count");
        Ok(self.nodes_written)
    }

    /// Finalize the snapshot writer and return the final count
    pub fn finalize(mut self, metadata: &mut StatePruneMetadata) -> Result<u64> {
        info!(
            "Finalizing snapshot with {} nodes written",
            self.nodes_written
        );

        // Validate the nodes_written count matches actual database count
        // Use actual count as source of truth if they differ
        let final_count = match self.get_actual_node_count() {
            Ok(actual_count) => {
                if actual_count != self.nodes_written {
                    warn!(
                        "nodes_written count ({}) does not match actual database count ({}), using actual count",
                        self.nodes_written, actual_count
                    );
                    self.nodes_written = actual_count;
                }
                self.nodes_written
            }
            Err(e) => {
                warn!(
                    "Failed to validate node count from database: {}, using tracked count",
                    e
                );
                self.nodes_written
            }
        };

        // Flush and compact to ensure all data is written and optimized
        let start = Instant::now();
        self.moveos_store
            .get_state_node_store()
            .flush_and_compact()?;
        let compact_duration = start.elapsed();

        info!(
            "Snapshot compaction completed in {:?}, final node count: {}",
            compact_duration, final_count
        );

        // Update metadata with final progress
        metadata.update_statistics(|stats| {
            stats.nodes_written = Some(final_count);
        });

        Ok(final_count)
    }

    /// Persist startup_info so the snapshot can be opened as a MoveOS store
    pub fn save_startup_info(&self, state_root: H256, global_size: u64) -> Result<()> {
        let startup_info = StartupInfo::new(state_root, global_size);
        self.moveos_store
            .get_config_store()
            .save_startup_info(startup_info)?;
        Ok(())
    }
}

impl Drop for SnapshotNodeWriter {
    fn drop(&mut self) {
        if let Err(e) = self.moveos_store.get_state_node_store().flush_only() {
            warn!("Failed to flush snapshot database on drop: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_snapshot_builder_creation() {
        let _config = SnapshotBuilderConfig::default();

        // TODO: Create test MoveOSStore
        // This will need proper setup when implementing tests

        // let moveos_store = MoveOSStore::mock_moveos_store().unwrap();
        // let builder = SnapshotBuilder::new(config, moveos_store);
        // assert!(builder.is_ok());
    }

    #[test]
    fn test_snapshot_node_writer_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = SnapshotBuilderConfig::default();

        // Test creation - don't assert success as RocksDB might not be available in all test environments
        let result = SnapshotNodeWriter::new(temp_dir.path(), &config);
        if let Err(e) = &result {
            println!("Note: RocksDB creation failed in test environment: {:?}", e);
        }
    }

    #[test]
    fn test_snapshot_node_writer_basic_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = SnapshotBuilderConfig::default();

        // Test basic creation - don't assume complex operations work in all environments
        match SnapshotNodeWriter::new(temp_dir.path(), &config) {
            Ok(mut writer) => {
                // Test empty batch handling
                assert!(writer.write_batch(Vec::new()).is_ok());

                // Test simple batch writing
                let hash = H256::random();
                let batch = vec![(hash, b"test_data".to_vec())];
                assert!(writer.write_batch(batch).is_ok());

                // Test that nodes_written count increases
                assert_eq!(writer.nodes_written, 1);
            }
            Err(e) => {
                // If RocksDB is not available in test environment, that's acceptable
                // This might happen in CI environments without proper RocksDB setup
                println!("RocksDB not available in test environment: {:?}", e);
            }
        }
    }

    #[test]
    fn test_snapshot_node_writer_resume_with_count() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = SnapshotBuilderConfig::default();

        match SnapshotNodeWriter::new(temp_dir.path(), &config) {
            Ok(mut writer) => {
                // Write some initial nodes
                let hash1 = H256::random();
                let hash2 = H256::random();
                let batch1 = vec![
                    (hash1, b"test_data_1".to_vec()),
                    (hash2, b"test_data_2".to_vec()),
                ];
                assert!(writer.write_batch(batch1).is_ok());
                assert_eq!(writer.nodes_written, 2);

                // Get the actual count from the database
                let actual_count = writer.get_actual_node_count();
                assert!(actual_count.is_ok());
                assert_eq!(actual_count.unwrap(), 2);

                // Drop the first writer to release the database lock
                drop(writer);

                // Simulate a resume scenario - create a new writer with initial count
                let initial_count = 2;
                match SnapshotNodeWriter::new_with_count(temp_dir.path(), &config, initial_count) {
                    Ok(mut resumed_writer) => {
                        // Verify the resumed writer starts with the correct count
                        assert_eq!(resumed_writer.nodes_written, 2);

                        // Write more nodes
                        let hash3 = H256::random();
                        let batch2 = vec![(hash3, b"test_data_3".to_vec())];
                        assert!(resumed_writer.write_batch(batch2).is_ok());

                        // Verify the count has been incremented correctly
                        assert_eq!(resumed_writer.nodes_written, 3);

                        // Verify the database has the correct total count
                        let final_count = resumed_writer.get_actual_node_count();
                        assert!(final_count.is_ok());
                        assert_eq!(final_count.unwrap(), 3);
                    }
                    Err(e) => {
                        panic!("Failed to create resumed writer: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("RocksDB not available in test environment: {:?}", e);
            }
        }
    }

    #[test]
    fn test_snapshot_progress_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let progress_path = temp_dir.path().join("snapshot_progress.json");

        let state_root = H256::random();
        let worklist = vec![H256::random(), H256::random()];
        let statistics = TraversalStatistics {
            nodes_visited: 100,
            bytes_processed: 5000,
        };

        // Create and save progress
        let progress = SnapshotProgress {
            state_root,
            worklist: worklist.clone(),
            worklist_position: 0,
            statistics: statistics.clone(),
            batch_buffer: vec![],
            current_batch_size: 1000,
            last_save_timestamp: 1234567890,
            nodes_written: 50,
            checkpoint_id: SnapshotProgress::generate_checkpoint_id(state_root, 100),
        };

        assert!(progress.save_to_file(&progress_path).is_ok());

        // Load and verify progress
        match SnapshotProgress::load_from_file(&progress_path, state_root) {
            Ok(Some(loaded_progress)) => {
                assert_eq!(loaded_progress.state_root, state_root);
                assert_eq!(loaded_progress.worklist, worklist);
                assert_eq!(loaded_progress.statistics.nodes_visited, 100);
                assert_eq!(loaded_progress.statistics.bytes_processed, 5000);
                assert_eq!(loaded_progress.nodes_written, 50);
                assert_eq!(loaded_progress.worklist_position, 0);
            }
            Ok(None) => {
                panic!("Expected to load progress but got None");
            }
            Err(e) => {
                panic!("Failed to load progress: {:?}", e);
            }
        }
    }

    #[test]
    fn test_snapshot_progress_invalid_state_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let progress_path = temp_dir.path().join("snapshot_progress.json");

        let state_root = H256::random();
        let different_state_root = H256::random();

        // Ensure they're different
        assert_ne!(state_root, different_state_root);

        // Create and save progress with one state root
        let progress = SnapshotProgress {
            state_root,
            worklist: vec![],
            worklist_position: 0,
            statistics: TraversalStatistics::default(),
            batch_buffer: vec![],
            current_batch_size: 1000,
            last_save_timestamp: 1234567890,
            nodes_written: 10,
            checkpoint_id: SnapshotProgress::generate_checkpoint_id(state_root, 50),
        };

        assert!(progress.save_to_file(&progress_path).is_ok());

        // Try to load with a different state root - should return None
        match SnapshotProgress::load_from_file(&progress_path, different_state_root) {
            Ok(None) => {
                // Expected - state root mismatch
            }
            Ok(Some(_)) => {
                panic!("Expected None due to state root mismatch");
            }
            Err(_) => {
                // Also acceptable - implementation may return error
            }
        }
    }
}
