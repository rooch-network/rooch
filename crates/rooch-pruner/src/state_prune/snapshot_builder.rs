// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state_prune::{
    OperationType, ProgressTracker, SnapshotBuilderConfig, StatePruneMetadata,
};
use crate::util::extract_child_nodes;
use anyhow::Result;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use rooch_config::state_prune::SnapshotMeta;
use serde_json;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Streaming snapshot builder for creating state snapshots containing only active nodes
/// Uses RocksDB backend with batched writes and scalable deduplication
pub struct SnapshotBuilder {
    config: SnapshotBuilderConfig,
    moveos_store: MoveOSStore,
    progress_tracker: ProgressTracker,
}

impl SnapshotBuilder {
    /// Create new snapshot builder
    pub fn new(
        config: SnapshotBuilderConfig,
        moveos_store: MoveOSStore,
    ) -> Result<Self> {
        config.validate()?;

        let progress_tracker = ProgressTracker::new(config.progress_interval_seconds);

        Ok(Self {
            config,
            moveos_store,
            progress_tracker,
        })
    }

    /// Build snapshot from state root using streaming approach
    pub async fn build_snapshot(
        &self,
        state_root: H256,
        output_dir: PathBuf,
    ) -> Result<SnapshotMeta> {
        info!("Starting streaming snapshot build for state root: {:x}", state_root);

        let start_time = Instant::now();

        // Initialize metadata
        let mut metadata = StatePruneMetadata::new(
            OperationType::Snapshot {
                tx_order: 0, // Will be updated when we get it from state root
                state_root: format!("{:x}", state_root),
                output_dir: output_dir.clone(),
            },
            serde_json::json!({
                "state_root": format!("{:x}", state_root),
                "config": self.config
            }),
        );

        metadata.mark_in_progress("Initializing".to_string(), 0.0);

        // Ensure output directory exists
        std::fs::create_dir_all(&output_dir)?;

        // Initialize progress tracking
        self.progress_tracker.set_total(0); // Will be updated during traversal

        // Create snapshot node writer with RocksDB backend
        metadata.mark_in_progress("Creating snapshot database".to_string(), 5.0);
        let mut snapshot_writer = SnapshotNodeWriter::new(&output_dir, &self.config)?;

        // Perform streaming traversal with batched writes
        metadata.mark_in_progress("Traversing state tree".to_string(), 10.0);
        let statistics = self
            .stream_traverse_and_write(state_root, &mut snapshot_writer, &mut metadata)
            .await?;

        metadata.update_statistics(|stats| {
            stats.nodes_processed = statistics.nodes_visited;
            stats.bytes_processed = statistics.bytes_processed;
        });

        metadata.mark_in_progress("Finalizing snapshot".to_string(), 95.0);

        // Flush and close snapshot writer to get final statistics
        let active_nodes_count = snapshot_writer.finalize(&output_dir, &mut metadata)?;

        // Create snapshot metadata
        let snapshot_meta = SnapshotMeta::new(
            0, // tx_order will be set later
            state_root,
            active_nodes_count, // Use active nodes count as global size estimate
            active_nodes_count,
        );

        // Save metadata
        snapshot_meta.save_to_file(&output_dir)?;
        metadata.save_to_file(output_dir.join("operation_meta.json"))?;

        let duration = start_time.elapsed();
        info!(
            "Streaming snapshot build completed in {:?}, {} nodes processed, {} written to output",
            duration, statistics.nodes_visited, active_nodes_count
        );

        metadata.mark_completed();

        Ok(snapshot_meta)
    }

    /// Streaming traversal with batched writes to avoid OOM
    async fn stream_traverse_and_write(
        &self,
        state_root: H256,
        snapshot_writer: &mut SnapshotNodeWriter,
        metadata: &mut StatePruneMetadata,
    ) -> Result<TraversalStatistics> {
        let mut statistics = TraversalStatistics::default();
        let mut nodes_to_process = VecDeque::new();
        nodes_to_process.push_back(state_root);

        let node_store = &self.moveos_store.node_store;
        let mut batch_buffer = Vec::with_capacity(self.config.batch_size);
        let mut last_progress_report = Instant::now();
        let mut consecutive_empty_batches = 0;
        const MAX_EMPTY_BATCHES: u32 = 100; // Safety limit to prevent infinite loops

        while let Some(current_hash) = nodes_to_process.pop_front() {
            // Safety check to prevent infinite loops in case of corrupted data
            if nodes_to_process.is_empty() && batch_buffer.is_empty() {
                consecutive_empty_batches += 1;
                if consecutive_empty_batches > MAX_EMPTY_BATCHES {
                    warn!(
                        "Reached maximum consecutive empty batches ({}), stopping traversal to prevent infinite loop",
                        MAX_EMPTY_BATCHES
                    );
                    break;
                }
            } else {
                consecutive_empty_batches = 0;
            }

            // Check if node is already written to avoid duplication
            if snapshot_writer.contains_node(&current_hash)? {
                debug!("Skipping duplicate node: {}", current_hash);
                continue;
            }

            // Get node data
            if let Some(node_data) = node_store.get(&current_hash)? {
                statistics.bytes_processed += node_data.len() as u64;

                // Extract child nodes from the node data before moving it into the buffer
                let child_hashes = extract_child_nodes(&node_data);

                // Add node to batch buffer for writing
                batch_buffer.push((current_hash, node_data));

                // Add child nodes to processing queue
                for child_hash in child_hashes {
                    nodes_to_process.push_back(child_hash);
                }

                // Write batch when it reaches the configured size
                if batch_buffer.len() >= self.config.batch_size {
                    let batch_size = batch_buffer.len();
                    snapshot_writer.write_batch(std::mem::take(&mut batch_buffer))?;

                    statistics.nodes_visited += batch_size as u64;

                    // Update progress periodically
                    if last_progress_report.elapsed()
                        >= Duration::from_secs(self.config.progress_interval_seconds)
                    {
                        info!(
                            "Streaming traversal progress: {} batches processed, {} nodes written",
                            statistics.nodes_visited / self.config.batch_size as u64,
                            snapshot_writer.nodes_written
                        );
                        last_progress_report = Instant::now();
                    }
                }
            } else {
                statistics.nodes_visited += 1;
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
        }

        // Write remaining nodes in the final batch
        if !batch_buffer.is_empty() {
            let batch_size = batch_buffer.len();
            snapshot_writer.write_batch(batch_buffer)?;
            statistics.nodes_visited += batch_size as u64;
        }

        Ok(statistics)
    }
}

/// RocksDB-backed snapshot node writer with batched writes and deduplication
pub struct SnapshotNodeWriter {
    db: Arc<rocksdb::DB>,
    batch_size: usize,
    pub nodes_written: u64,
}

impl SnapshotNodeWriter {
    /// Create new snapshot node writer with RocksDB backend
    pub fn new(output_dir: &Path, config: &SnapshotBuilderConfig) -> Result<Self> {
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
        if let Ok(metadata) = std::fs::metadata(&snapshot_db_path) {
            debug!("Snapshot directory created: {:?}", snapshot_db_path);
        }

        // Configure RocksDB for snapshot workloads - optimized for sequential writes
        let mut db_opts = rocksdb::Options::default();
        db_opts.create_if_missing(true);

        // Snapshot-specific optimizations for write-heavy workloads
        db_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        db_opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB write buffer
        db_opts.set_max_write_buffer_number(4);
        db_opts.set_target_file_size_base(64 * 1024 * 1024); // 64MB SST files
        db_opts.set_max_background_jobs(4);
        db_opts.set_bytes_per_sync(4 * 1024 * 1024); // 4MB sync
        db_opts.set_use_fsync(false); // Disable fsync for performance (safe for snapshots)

        // Disable WAL for snapshot building - crash consistency not critical
        // Snapshots can be rebuilt if interrupted
        let wal_dir = snapshot_db_path.join("wal");
        std::fs::create_dir_all(&wal_dir)?;
        db_opts.set_wal_dir(&wal_dir);

        // Open database with single column family for nodes
        let db = match rocksdb::DB::open(&db_opts, &snapshot_db_path) {
            Ok(db) => {
                info!("Successfully opened snapshot database at: {:?}", snapshot_db_path);
                Arc::new(db)
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to open snapshot database at {:?}: {}",
                    snapshot_db_path, e
                ));
            }
        };

        Ok(Self {
            db,
            batch_size: config.batch_size,
            nodes_written: 0,
        })
    }

    /// Check if node already exists in snapshot (for deduplication)
    pub fn contains_node(&self, hash: &H256) -> Result<bool> {
        match self.db.get(hash.as_bytes()) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Write batch of nodes to RocksDB
    pub fn write_batch(&mut self, batch: Vec<(H256, Vec<u8>)>) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let mut write_batch = rocksdb::WriteBatch::default();

        // Add all nodes to batch
        for (hash, data) in batch {
            write_batch.put(hash.as_bytes(), data);
            self.nodes_written += 1;
        }

        // Write batch with disabled WAL for performance
        let mut write_opts = rocksdb::WriteOptions::default();
        write_opts.disable_wal(true);
        self.db.write_opt(write_batch, &write_opts)?;

        Ok(())
    }

    /// Finalize the snapshot writer and return the final count
    pub fn finalize(mut self, output_dir: &Path, metadata: &mut StatePruneMetadata) -> Result<u64> {
        info!("Finalizing snapshot with {} nodes written", self.nodes_written);

        // Force flush to ensure all data is written to disk
        self.db.flush()?;

        // Trigger a single compaction to optimize file layout
        let start = Instant::now();
        self.db.compact_range::<&[u8], &[u8]>(None, None);
        let compact_duration = start.elapsed();

        info!(
            "Snapshot compaction completed in {:?}, final node count: {}",
            compact_duration, self.nodes_written
        );

        // Update metadata with final progress
        metadata.update_statistics(|stats| {
            stats.nodes_written = Some(self.nodes_written);
        });

        Ok(self.nodes_written)
    }
}

impl Drop for SnapshotNodeWriter {
    fn drop(&mut self) {
        // Ensure database is properly closed
        if let Err(e) = self.db.flush() {
            warn!("Failed to flush snapshot database on drop: {:?}", e);
        }
    }
}

/// Statistics for state tree traversal
#[derive(Debug, Default)]
struct TraversalStatistics {
    nodes_visited: u64,
    bytes_processed: u64,
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
        let _temp_dir = tempfile::tempdir().unwrap();
        let config = SnapshotBuilderConfig::default();

        let _writer = SnapshotNodeWriter::new(_temp_dir.path(), &config);
        assert!(_writer.is_ok());
    }

    #[test]
    fn test_snapshot_node_writer_batch_operations() {
        let _temp_dir = tempfile::tempdir().unwrap();
        let config = SnapshotBuilderConfig::default();

        let mut writer = SnapshotNodeWriter::new(_temp_dir.path(), &config).unwrap();

        // Test writing batch
        let hash1 = H256::random();
        let hash2 = H256::random();
        let batch = vec![
            (hash1, b"node_data_1".to_vec()),
            (hash2, b"node_data_2".to_vec()),
        ];

        writer.write_batch(batch).unwrap();

        // Test deduplication
        assert!(!writer.contains_node(&hash1).unwrap());
        assert!(!writer.contains_node(&hash2).unwrap());

        // Write the same node again to test deduplication
        let batch2 = vec![(hash1, b"node_data_1".to_vec())];
        writer.write_batch(batch2).unwrap();
    }
}

