// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state_prune::{
    OperationType, ProgressTracker, SnapshotBuilderConfig, StatePruneMetadata,
};
use crate::util::extract_child_nodes;
use anyhow::Result;
use moveos_types::h256::H256;
use rooch_config::state_prune::SnapshotMeta;
use serde_json;
use smt::NodeReader;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

/// Snapshot builder for creating state snapshots containing only active nodes
pub struct SnapshotBuilder {
    config: SnapshotBuilderConfig,
    moveos_store: moveos_store::MoveOSStore,
    progress_tracker: ProgressTracker,
}

impl SnapshotBuilder {
    /// Create new snapshot builder
    pub fn new(
        config: SnapshotBuilderConfig,
        moveos_store: moveos_store::MoveOSStore,
    ) -> Result<Self> {
        config.validate()?;

        let progress_tracker = ProgressTracker::new(config.progress_interval_seconds);

        Ok(Self {
            config,
            moveos_store,
            progress_tracker,
        })
    }

    /// Build snapshot from state root
    pub async fn build_snapshot(
        &self,
        state_root: H256,
        output_dir: PathBuf,
    ) -> Result<SnapshotMeta> {
        info!("Starting snapshot build for state root: {:x}", state_root);

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

        // Create bloom filter if enabled
        let bloom_filter = if self.config.enable_bloom_filter {
            Some(Arc::new(BloomFilter::new(self.config.bloom_filter_fp_rate)))
        } else {
            None
        };

        // Collect all active nodes
        metadata.mark_in_progress("Traversing state tree".to_string(), 10.0);
        let (active_nodes, statistics) = self
            .traverse_active_nodes(state_root, bloom_filter.clone(), &metadata)
            .await?;

        metadata.update_statistics(|stats| {
            stats.nodes_processed = statistics.nodes_visited;
            stats.bytes_processed = statistics.bytes_processed;
        });

        metadata.mark_in_progress("Saving snapshot data".to_string(), 80.0);

        // Save nodes to output database
        let active_nodes_count = active_nodes.len() as u64;
        self.save_nodes_to_snapshot(active_nodes, &output_dir, &mut metadata)
            .await?;

        // Create snapshot metadata
        let snapshot_meta = SnapshotMeta::new(
            0, // tx_order will be set later
            state_root,
            statistics.global_size,
            active_nodes_count,
        );

        // Save metadata
        let _metadata_path = output_dir.join("snapshot_meta.json");
        snapshot_meta.save_to_file(&output_dir)?;
        metadata.save_to_file(output_dir.join("operation_meta.json"))?;

        metadata.mark_in_progress("Finalizing".to_string(), 95.0);

        let duration = start_time.elapsed();
        info!(
            "Snapshot build completed in {:?}, {} nodes processed",
            duration, active_nodes_count
        );

        metadata.mark_completed();

        Ok(snapshot_meta)
    }

    /// Traverse state tree and collect active nodes
    async fn traverse_active_nodes(
        &self,
        state_root: H256,
        bloom_filter: Option<Arc<BloomFilter>>,
        _metadata: &StatePruneMetadata,
    ) -> Result<(BTreeMap<H256, Vec<u8>>, TraversalStatistics)> {
        let mut active_nodes = BTreeMap::new();
        let mut visited_nodes = HashSet::new();
        let mut nodes_to_process = vec![state_root];
        let mut statistics = TraversalStatistics::default();

        let node_store = &self.moveos_store.node_store;

        while let Some(current_hash) = nodes_to_process.pop() {
            // Skip if already visited
            if !visited_nodes.insert(current_hash) {
                continue;
            }

            // Check bloom filter if enabled
            if let Some(ref filter) = bloom_filter {
                if filter.contains(&current_hash) {
                    continue;
                }
                filter.insert(&current_hash);
            }

            // Get node data
            if let Some(node_data) = node_store.get(&current_hash)? {
                statistics.bytes_processed += node_data.len() as u64;
                active_nodes.insert(current_hash, node_data);

                // Extract child nodes and add to processing queue
                self.extract_child_nodes(&current_hash, &mut nodes_to_process)?;
            }

            statistics.nodes_visited += 1;

            // Update progress periodically
            if self.progress_tracker.should_report() {
                let progress = self.progress_tracker.get_progress_report();
                info!("Traversal progress: {}", progress.format());
                self.progress_tracker.mark_reported();
            }
        }

        Ok((active_nodes, statistics))
    }

    /// Extract child nodes from current node
    #[allow(clippy::ptr_arg)]
    fn extract_child_nodes(
        &self,
        parent_hash: &H256,
        nodes_to_process: &mut Vec<H256>,
    ) -> Result<()> {
        // Get node data from the store
        if let Some(node_data) = self.moveos_store.node_store.get(parent_hash)? {
            // Extract child nodes using the existing utility function
            let child_hashes = extract_child_nodes(&node_data);

            // Add all child nodes to processing queue
            for child_hash in child_hashes {
                nodes_to_process.push(child_hash);
                debug!("Added child node {} for processing", child_hash);
            }
        }

        Ok(())
    }

    /// Save nodes to snapshot database
    async fn save_nodes_to_snapshot(
        &self,
        nodes: BTreeMap<H256, Vec<u8>>,
        output_dir: &Path,
        metadata: &mut StatePruneMetadata,
    ) -> Result<()> {
        let snapshot_db_path = output_dir.join("snapshot.db");

        info!("Saving {} nodes to snapshot database", nodes.len());

        // Create snapshot database
        let snapshot_store = self.create_snapshot_store(&snapshot_db_path)?;

        // Save nodes in batches
        let mut saved_count = 0;
        let total_nodes = nodes.len();

        for (i, (hash, data)) in nodes.into_iter().enumerate() {
            snapshot_store.put(hash, data)?;
            saved_count += 1;

            // Update progress
            if i % self.config.batch_size == 0 {
                let progress = 80.0 + (saved_count as f64 / total_nodes as f64) * 15.0;
                metadata.mark_in_progress(
                    format!("Saving nodes ({}/{})", saved_count, total_nodes),
                    progress,
                );

                info!("Saved {}/{} nodes", saved_count, total_nodes);
            }
        }

        info!("Successfully saved {} nodes to snapshot", saved_count);

        Ok(())
    }

    /// Create snapshot store (simplified file-based implementation)
    fn create_snapshot_store(&self, output_dir: &Path) -> Result<Box<dyn NodeStore>> {
        let nodes_dir = output_dir.join("nodes");
        std::fs::create_dir_all(&nodes_dir)?;
        Ok(Box::new(FileNodeStore::new(nodes_dir)))
    }
}

/// Statistics for state tree traversal
#[derive(Debug, Default)]
struct TraversalStatistics {
    nodes_visited: u64,
    global_size: u64,
    bytes_processed: u64,
}

/// Bloom filter implementation (simplified)
#[allow(dead_code)]
struct BloomFilter {
    fp_rate: f64,
    // TODO: Implement actual bloom filter data structure
}

impl BloomFilter {
    fn new(fp_rate: f64) -> Self {
        Self { fp_rate }
    }

    fn contains(&self, _hash: &H256) -> bool {
        // TODO: Implement actual bloom filter check
        false
    }

    fn insert(&self, _hash: &H256) {
        // TODO: Implement actual bloom filter insertion
    }
}

/// Node store trait
trait NodeStore {
    fn put(&self, key: H256, value: Vec<u8>) -> Result<()>;
}

/// Simple file-based snapshot node store
struct FileNodeStore {
    nodes_dir: PathBuf,
}

impl FileNodeStore {
    fn new(nodes_dir: PathBuf) -> Self {
        Self { nodes_dir }
    }

    fn node_file_path(&self, key: H256) -> PathBuf {
        self.nodes_dir
            .join(format!("{}.bin", hex::encode(key.as_bytes())))
    }
}

impl NodeStore for FileNodeStore {
    fn put(&self, key: H256, value: Vec<u8>) -> Result<()> {
        let file_path = self.node_file_path(key);
        std::fs::write(file_path, value)?;
        Ok(())
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
}
